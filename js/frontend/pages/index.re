[@react.component]
let make () = {
  open PictureUpload;
  let (state, setFile) = usePictureUpload();
  let onFileChange(event) = setFile((_) => FileHooks.getFiles(event));
  <Layout>{
    switch state {
    | Wait(prevError) =>
      <form>
        {
          switch prevError {
          | None => React.null
          | Some(err) =>
            <p className="error">{ React.string(err) }</p>
          }
        }
        <input type_="file" onChange=onFileChange />
      </form>
    | Loading => <p>{ R.t("inputPicture.loading") }</p>
    | Finished(uri) =>
      <div>
        <img src=uri />
      </div>
    }
  }</Layout>;
};

let default = make;
