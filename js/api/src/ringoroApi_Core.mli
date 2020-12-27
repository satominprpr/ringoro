exception UrlBaseIsNotGiven
exception ResponceNotOk of Js.Json.t

val urlBase: string option ref
val headerAdditional: (string * string) array ref
val call: string -> 'a -> 'b Js.Promise.t
