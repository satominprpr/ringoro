module Types = RingoroApi_Types
module Core = RingoroApi_Core

let getUser: Types.UserFindOneInput.t -> Types.UserOutput.t option Js.Promise.t = Core.call "user"

let getUsers: unit -> Types.UserOutput.t array option Js.Promise.t = fun () -> Core.call "users" Js.Null.empty
