open Jest;
open Expect;
open RingoroApi;
open RingoroApi.Types;
open Js.Promise;

[@bs.module "process"]
external env: Js.Dict.t(string) = "env";

[@bs.module "./mongo_util"]
external resetDb: unit => Js.Promise.t(unit) = "resetDb";

let user: ref(option(UserOutput.t)) = ref(None);

beforeAll(() => {
  Core.urlBase := Some(env->Js.Dict.unsafeGet("SERVER_ROOT_URL") ++ "/api");
});

beforeEachPromise(() => {
  resetDb()
});

let createUser = (~admin=false, username) => {
  open Fetch;
  let url = env->Js.Dict.unsafeGet("SERVER_ROOT_URL");
  let path =
    if (admin) {
      "/test_create_user?admin=true&username=" ++ username
    } else {
      "/test_create_user?admin=false&username=" ++ username
    };
  fetch(url ++ path)
    |> then_(res => {
        let cookie =
          res
            |> Response.headers
            |> Headers.get("set-Cookie")
            |> Belt.Option.getExn;
        let cookie = Js.Array.unsafe_get(Js.String.split(";", cookie), 0);
        Core.headerAdditional := [| ("Cookie", cookie) |];
        resolve()
      })
};

[@bs.scope("JSON")][@bs.val]
external parseInto: string => 'a = "parse";

describe("/api/user", () => {
  testPromise("get with normal user", () => {
    let data = UserFindOneInput.empty;
    createUser("akari")
      |> then_(() => getUser(data))
      |> then_(user => {
        let user = Belt.Option.getExn(user);
        expect(user.UserOutput.name) |> toEqual("akari")
          |> resolve
      })
  });

  testPromise("get with admin user", () => {
    let data = UserFindOneInput.empty;
    createUser(~admin=true, "akari")
      |> then_(() => getUser(data))
      |> then_(user => {
        let user = Belt.Option.getExn(user);
        expect(user.UserOutput.name) |> toEqual("akari")
          |> resolve
      })
  });

  testPromise("fail with nologin", () => {
    let data = UserFindOneInput.empty;
    getUser(data)
      |> then_(_ => { fail("don't success") |> resolve })
      |> catch(_ => { pass |> resolve })
  });
});

describe("/api/users", () => {
  testPromise("get with admin user", () => {
    createUser(~admin=true, "akari")
      |> then_(() => getUsers())
      |> then_(user => {
        let user = Belt.Option.getExn(user);
        expect(Js.Array.length(user)) |> toEqual(1)
          |> resolve
      })
  });

  testPromise("fail with normal user", () => {
    createUser("akari")
      |> then_(() => getUsers())
      |> then_(_ => { fail("don't success") |> resolve })
      |> catch(_ => { pass |> resolve })
  });

  testPromise("fail with nologin", () => {
    getUsers()
      |> then_(_ => { fail("don't success") |> resolve })
      |> catch(_ => { pass |> resolve })
  });
});
