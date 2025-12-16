import NextAuth from "next-auth"
import { MongoDBAdapter } from "@auth/mongodb-adapter"
import clientPromise from "@/lib/db/adapter-client"
import authConfig from "@/auth.config"

export const { handlers, signIn, signOut, auth } = NextAuth({
    adapter: MongoDBAdapter(clientPromise),
    session: { strategy: "database" },
    ...authConfig,
})
