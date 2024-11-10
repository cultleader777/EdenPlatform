type consul_update = {
  key: string [@key "Key"];
  create_index: int [@key "CreateIndex"];
  modify_index: int [@key "ModifyIndex"];
  lock_index: int [@key "LockIndex"];
  flags: int [@key "Flags"];
  value: string [@key "Value"];
  session: string [@key "Session"];
} [@@deriving yojson]

let main () =
  let arr = Sys.argv in
  if Array.length arr != 4 then (
    prerr_endline "Expect three arguments: path to our private key file, path to public key file, path to the temp buffer file";
    exit 7;
  ) else (
    let our_private_key_file = arr.(1) in
    let authority_public_key_file = arr.(2) in
    let decrypt_file = arr.(3) in
    let our_private_key =
      In_channel.with_open_bin our_private_key_file In_channel.input_all
      |> String.trim |> Base64.decode_exn |> Bytes.of_string |> Sodium.Box.Bytes.to_secret_key in
    let auth_public_key =
      In_channel.with_open_bin authority_public_key_file In_channel.input_all
      |> String.trim |> Base64.decode_exn |> Bytes.of_string |> Sodium.Box.Bytes.to_public_key in
    (* message format *)
    (* [nonce][encrypted payload]*)
    let payload_json = In_channel.stdin
      |> In_channel.input_all
      |> (* decode base64 from consul stdin*) String.trim in
    let payload_parsed = Yojson.Safe.from_string payload_json |> consul_update_of_yojson in
    match payload_parsed with
    | Ok parsed -> (
      let db = Sqlite3.db_open "/var/lib/epl-l1-prov/provisionings.sqlite" in
      (* no worries about sql injection because number is a number *)
      let insert_code = Sqlite3.exec db (Printf.sprintf {|
        INSERT INTO consul_l1_payloads(consul_modify_index) VALUES(%d)
      |} parsed.modify_index) in
      let _ = Sqlite3.db_close db in
      if insert_code = Sqlite3.Rc.OK then (
        let payload = parsed.value |> Base64.decode_exn |> Bytes.of_string in
        Printf.printf "Received payload length %d\n" (Bytes.length payload);
        let nonce = Bytes.sub payload 0 Sodium.Box.nonce_size |> Sodium.Box.Bytes.to_nonce in
        let encrypted_payload = Bytes.sub payload Sodium.Box.nonce_size (Bytes.length payload - Sodium.Box.nonce_size) in
        let decrypted_bytes = Sodium.Box.Bytes.box_open our_private_key auth_public_key encrypted_payload nonce in
        Out_channel.with_open_gen [Open_creat; Open_wronly; Open_binary] 0o600 decrypt_file (fun oc ->
            Out_channel.output_bytes oc decrypted_bytes;
          );
        let exit_code = Sys.command (Printf.sprintf "unzstd -c %s | /bin/sh --login; rm -f %s" decrypt_file decrypt_file) in
        if exit_code <> 0 then (
          Printf.sprintf "Failed executing decrypted command, exit code: %d" exit_code |> prerr_endline;
        ) else (
          print_endline "Execution successful"
        )
      ) else (
        Printf.sprintf "Consul modify index %d was already seen, skipping" parsed.modify_index |> prerr_endline
      )
    )
    | Error err -> (
        prerr_endline err
    )
  )

let () =
  Printexc.record_backtrace true;
  try main () with
  | exn -> (
    (* Never exit with non-zero, ignore failures not to restart calling consul service*)
    Printf.sprintf "Error evaluating fast l1 payload: %s" (Printexc.to_string exn) |> prerr_endline
  )
