module R = Resource

type state = Wait of string option | Loading | Finished of string

let usePictureUpload () =
  let availableTypes = [|
    "image/png";
    "imaga/jpeg";
    "image/svg+xml"
  |] in
  (*let maxSize = 1 * 1024 * 2014;*)
  let fileValidate fileinfo =
    let open FileHooks.FileInfo in
    let mimetype = fileinfo |> type_ in
    if availableTypes |> Js.Array.includes mimetype then
      Ok ()
    else
      Error (R.ts "errors.picureUploadInvalidFileType") in
  let fileState, resetFileState, setFile =
    FileHooks.useFileInfoGetter () in
  let state, setState = React.useState (fun () -> Wait None) in
  let proc () =
    let raiseError err =
      setState (fun _ -> Wait (Some err));
      resetFileState () in
    match fileState with
    | FileHooks.Wait -> ()
    | FileHooks.Loading -> setState (fun _ -> Loading)
    | FileHooks.Finished Ok([| file |]) -> begin
        match fileValidate file with
        | Ok () ->
            let uri = FileHooks.FileInfo.toUri file in
            setState (fun _ -> Finished uri)
        | Error err -> raiseError err
    end
    | FileHooks.Finished Ok(_) -> raiseError "unexpected"
    | FileHooks.Finished(Error(_err)) -> raiseError "error" in
  React.useEffect1 (fun () -> proc(); None) [| fileState |];
  state, setFile
