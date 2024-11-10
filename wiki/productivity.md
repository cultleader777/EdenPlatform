# Productivity

"Eden Platform - The most productive web development platform of all time"

A bold statement. Let's quickly define what we mean and what we don't mean by saying productivity.

What we don't mean by productivity:
- Writing shitty Ruby on Rails apps that have zero typesafety and productivity of an organization using Ruby on Rails quickly grinds to a halt because people need to start praying before doing changes. Undefined methods/variables errors everywhere in a big monolithic app. Problem becomes much worse with many ruby services talking together. Ruby fags claim this is remedied by tests and you better pray that people write tests (spoiler alert - they don't. at least not to the extent of correctness that statically typed languages give you). And before any big change you need to invite Ruby faggots to meetings to make sure that tens of subsystems will not break down after removing fields in a database or migrating database shard because all that information is not inside the compiler but in a Ruby faggots fallible mind.
- Writing lots of typesafe boilerplate code for Rust so that your contracts are correct. But even then, if you're using a database or other service in your ecosystem you need nasty error handling, like is this json parsed correctly.

These are two sides of the same coin, Ruby on Rails fags like to pretend that dynamic typing errors don't exist or they're not that bad. I wouldn't trust a Ruby fag to write any more mission critical system than a todo list. Rustaceans realized writing websites with dynamically typed languages was unproductive and error prone ages ago, but they face other issues, they need to define lots of boilerplate types and then they can use structs in Rust in a typesafe manner in their systems.

What we mean by productivity:
- If you have thousand of services in your infrastructure and you change type signature of a single REST endpoint you'll have to fix all the compile times errors everywhere that this service was used, because services that depend on this service will have their usage code of the changed service regenerated which will turn into compile time errors. You cannot ship until you fix all of that.
- If you drop a column in a database you'll have to fix every singe query or service that used that column across thousands of services. You cannot ship until you fix that.

And many more examples with queues, frontend queries and etc. that follow the same pattern - fix inconsistencies and compile time errors across your entire distributed system of thousands of services and thousands of servers or you cannot ship your code.

Eden platform generates a lot of code to make this happen. We enjoy having a terse code (10x less from vanilla Rust) because things like database access simply turn into typesafe functions of tested queries in Eden platform that we already know work and that must hit indexes, because there must be at least one test written for database query. We enjoy typesafe, backwards compatible queues in our entire infrastructure the same way also and don't worry about manual parsing of queue messages.

We enjoy terseness of code, which is boasted by Ruby on Rails faggots, but we also enjoy typesafety of Rust across thousands of services. And unlike Ruby on Rails, which generates instant legacy code which then becomes your problem to maintain, Eden platform assumes responsibility over generated code. If you remove database query from Eden data language, that query will disappear from the generated code across all the applications that it was used in, generating compile time errors which you must fix before you can ship.

That is what we mean by productivity. Always having consistency across distributed system of thousands of servers and thousands of services. And as far as I'm aware no framework in the world has even attempted to do this (and boy is it a lot of work to implement the cloud from scratch with typesafety and consistency checks across all services, believe me).

And that is why I claim Eden platform is the most productive web development platform of all time without any competition at all. That is, until other people will start copying what Eden platform has accomplished.
