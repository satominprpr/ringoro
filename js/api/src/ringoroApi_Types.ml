module Id: sig
  type t
end = struct
  type t = string
end

module UserOutput = struct
  type t = {
    id: Id.t;
    name: string;
  }
end

module UserFindOneInput = struct
  type t = {
    id: Id.t Js.Null.t;
  }

  let create id = { id=Js.Null.fromOption id}
  let empty = create None
end
