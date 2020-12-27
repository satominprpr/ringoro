module Resource = struct
  type t = {
    fonts: string Js.Dict.t [@bs.optional];
    contents: (string Js.Dict.t) Js.Dict.t [@bs.optional];
  } [@@bs.deriving {abstract = light}]
end

external _resource: Resource.t = "./resource.yaml" [@@bs.module]

let _parse resource =
  let open Resource in
  let fonts = resource |. fonts
  and contents = resource |. contents in
  fonts, contents

let _normalize_contents = fun [@bs] orig ->
  let f' pkey (ckey, value) = (pkey ^ "." ^ ckey), value in
  let f (pkey, cdic) =
    cdic
      |> Js.Dict.entries
      |> Array.map @@ f' pkey in
  try
    Js.Dict.entries orig
      |> Array.map f
      |> Array.to_list
      |> Array.concat
      |> Js.Dict.fromArray
      |> Js.Option.some
  with _ -> None

let _fonts, _resources =
  let open Js.Option in
  let fonts, resources = _parse _resource in
  fonts, andThen _normalize_contents resources

let ts key =
  let default = "\"" ^ key ^ "\" is not defined." in
  match _resources with
  | Some resources ->
      Js.Dict.get resources key
        |> Js.Option.getWithDefault default
  | None -> "resource file is not valid."

let t key = ts key |> React.string

let fonts =
  let _fonts = Js.Option.getExn _fonts in
  let default =
    Js.Dict.get _fonts "default"
      |> Js.Option.getWithDefault "sans-serif" in
  let search_buf =
    _fonts
      |> Js.Dict.entries
      |> Array.to_list
      |> List.filter (fun (k, _) -> k != "default")
      |> List.rev_map
          (fun (k, v) ->
            let re = 
              try
                let k = Js.String.replace "*" ".*" k in
                let regs = "^" ^ k in
                Js.Option.some @@ Js.Re.fromString regs
              with _ -> None in
            re, v)
      |> List.filter (fun (re, _) -> Js.Option.isSome re) in
  let search search_buf key =
    let result =
      search_buf
        |> List.filter (fun (re, _) ->
            Belt.Option.flatMap re (fun re -> Js.Re.exec_ re key)
              |> Js.Option.isSome)
        |> List.map (fun (_, font) -> font) in
    match result with
    | [] -> [default]
    | l -> l in
  let to_assoc orig (key, text) =
    let open Belt.List in
    let add text orig font =
      let old =
        getAssoc orig font (=)
          |> Js.Option.getWithDefault [] in
      setAssoc orig font (text :: old) (=) in
    let add_all text orig fonts =
      List.fold_left (add text)orig fonts in
    let fonts = search search_buf key in
    add_all text orig fonts in
  match _resources with
  | Some resources ->
    Js.Dict.entries resources
      |> Array.to_list
      |> List.fold_left to_assoc []
      |> List.map (fun (v, k) -> v, String.concat "" k)
  | None -> failwith "resource file is not valid."

let fontFamilyValue key =
  match _fonts with
  | Some fonts ->
    Js.Dict.get fonts key
  | None -> failwith "resource file is not valid."
