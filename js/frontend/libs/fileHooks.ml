module File = Webapi.File

module FileList: sig
  type t
  val toArray: t -> File.t array
end = struct
  type t

  external length: t -> int = "length"
  [@@bs.get]

  external item: t -> int -> File.t = "item"
  [@@bs.send]

  let toArray v =
    let rec f n l =
      if n = length v then l
      else f (n + 1) (item v n :: l) in
    f 0 [] |> Belt.List.toArray
end

module HasFilesElement = struct
  type t

  external toHasFilesElement_unsafe: 'a -> t = "%identity"

  external files: t -> FileList.t option = "files"
  [@@bs.get][@@bs.return(nullable)]
end

let getFiles event =
  let open ReactEvent in
  let elem = toSyntheticEvent event |> Synthetic.target in
  let elem = HasFilesElement.toHasFilesElement_unsafe elem in
  let result =
    Belt.Option.map (HasFilesElement.files elem) FileList.toArray in
  result

external arrayBufferToBase64: Js.Typed_array.ArrayBuffer.t -> string
  = "arrayBufferToBase64"
[@@bs.module "./base64.js"]

module FileInfo = struct
  type data =
    | Binary of Js.Typed_array.ArrayBuffer.t
    | Text of string

  type t = {
    data: data;
    type_: string;
  }

  let fromFile f =
    let open Js.String2 in
    let open Js.Promise in
    let open Webapi.File in
    let mimetype = f |> type_ in
    if (startsWith mimetype "text/") || (endsWith mimetype "+xml") then
      f |> text
        |> then_ (fun text -> resolve { data=Text text; type_=mimetype })
    else
      f |> arrayBuffer
        |> then_ (fun buf -> resolve { data=Binary buf; type_=mimetype })

  let toUri { data; type_ } =
    let cdata =
      match data with
      | Binary b -> ";Base64," ^ arrayBufferToBase64 b
      | Text t -> "," ^ Js.Global.encodeURIComponent t in
    "data:" ^ type_ ^ cdata

  let data { data } = data
  let type_ { type_ } = type_
end

type state =
  | Wait
  | Loading
  | Finished of (FileInfo.t Js.Array.t, Js.Promise.error) result

let useFileInfoGetter () =
  let open Js.Promise in
  let state, setState = React.useState (fun () -> Wait) in
  let files, setFiles = React.useState (fun () -> None) in
  let fileEffect () =
    match state, files with
    | Wait, Some files ->
        setState (fun _ -> Loading);
        Belt.Array.map files FileInfo.fromFile
          |> all
          |> then_ (fun files -> resolve @@ Finished (Ok files))
          |> catch (fun err ->
              Js.log err;
              resolve @@ Finished (Error err))
          |> then_ (fun state -> resolve @@ setState (fun _ -> state))
          |> ignore
    | _ -> () in
  let () = 
    React.useEffect2 (fun () -> fileEffect (); None) (files, state) in
  let resetState () =
    setFiles (fun _ -> None);
    setState (fun _ -> Wait) in
  state, resetState, setFiles
