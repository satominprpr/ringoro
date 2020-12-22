module File = Webapi.File

module FileInfo: sig
  type t
  type data =
    | Binary of Js.Typed_array.ArrayBuffer.t
    | Text of string

  val fromFile: File.t -> t Js.Promise.t
  val toUri: t -> string
  val data: t -> data
  val type_: t -> string
end

type state =
  | Wait
  | Loading
  | Finished of (FileInfo.t array, Js.Promise.error) result

val getFiles: 'a ReactEvent.synthetic -> Webapi.File.t array option
val useFileInfoGetter:
  unit ->
    state *
    (unit -> unit) *
    ((File.t array option -> File.t array option) -> unit)
