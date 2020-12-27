exception UrlBaseIsNotGiven
exception ResponceNotOk of Js.Json.t

[%%raw "require('isomorphic-fetch')"]

let urlBase: string option ref = ref None
let headerAdditional: (string * string) array ref = ref [||]

external parseInto: string -> 'a = "parse"
[@@bs.scope("JSON")][@@bs.val]

let fetchInit data =
  let open Fetch in
  let body =
    Js.Json.stringifyAny data
      |> Belt.Option.getExn
      |> BodyInit.make in
  let headers =
    Js.Array.concat !headerAdditional
      [|
        "Content-Type", "application/json"
      |]
    |> HeadersInit.makeWithArray in
  RequestInit.make
    ~method_:Post
    ~mode:SameOrigin
    ~credentials:SameOrigin
    ~headers
    ~body ()

let call path data =
  let open Js.Promise in
  let open Fetch in
  let url =
    match !urlBase with
    | Some base -> base ^ "/" ^ path
    | None -> raise UrlBaseIsNotGiven in
  let init = fetchInit data in
  fetchWithInit url init
    |> then_ begin fun result ->
        if Response.ok result then
          Response.text result
            |> then_ (fun body -> body |> parseInto |> resolve)
        else
          Response.json result
            |> then_ (fun json -> ResponceNotOk json |> reject)
    end
