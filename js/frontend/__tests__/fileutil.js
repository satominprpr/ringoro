import path from 'path'
import jsdom from 'jsdom'
import * as fs from "fs"
import * as mime from "mime-types"
const { JSDOM } = jsdom;

export const files = {
  jpg: path.join(__dirname, 'files/lena.jpg'),
  png: path.join(__dirname, 'files/lena.png'),
  svg: path.join(__dirname, 'files/file_example_SVG_20kB.svg'),
}

export function getFile(filepath) {
  const lastModified = fs.statSync(filepath).mtimeMs
  return new window.File(
    [fs.readFileSync(filepath)],
    path.basename(filepath),
    {
      lastModified,
      type: mime.lookup(filepath) || '',
    }
  )
}

export function fileToBin(path) {
  return fs.readFile(path)
}
