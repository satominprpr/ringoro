[@react.component]
let make(~children) = {
  <div>
    { children }
  </div>;
};

let default = make;
