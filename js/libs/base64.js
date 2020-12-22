export function arrayBufferToBase64(buf) {
  var tmp = [];
  var len = 1024;
  let buffer_to_string =
    (buf) => String.fromCharCode(...new Uint8Array(buf))
  for (var p = 0; p < buf.byteLength; p += len) {
    tmp.push(buffer_to_string(buf.slice(p, p + len)));
  }
  return btoa(tmp.join(""))
}

