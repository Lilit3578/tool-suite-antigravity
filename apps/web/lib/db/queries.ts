import dbConnect from "@/lib/db/connect";
import { User } from "@/lib/db/models";

export async function getUserByEmail(email: string) {
    try {
        await dbConnect();
        const user = await User.findOne({ email });
        return user;
    } catch (error) {
        console.error("Failed to fetch user:", error);
        throw new Error("Failed to fetch user.");
    }
}
