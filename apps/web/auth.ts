import NextAuth from "next-auth"
import { MongoDBAdapter } from "@auth/mongodb-adapter"
import clientPromise from "@/lib/db/adapter-client"
import authConfig from "@/auth.config"

export const { handlers, signIn, signOut, auth } = NextAuth({
    adapter: MongoDBAdapter(clientPromise),
    session: {
        strategy: "jwt",
        maxAge: 180 * 24 * 60 * 60, // 180 days
    },
    ...authConfig,
})
