import { MongoClient } from 'mongodb'

const url = process.env["RINGORO_DB_URI"]
const dbName = process.env["RINGORO_DB_DATABASE"]

export async function resetDb() {
  let client
  try {
    client = await MongoClient.connect(
      url, {
        useNewUrlParser: true
      })
    const db = client.db(dbName)
    await db.dropDatabase()
  } catch (error) {
    throw error
  } finally {
    if (client) client.close()
  }
}
