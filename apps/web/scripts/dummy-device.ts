
import { auth } from "@/auth"
import { User } from "@/lib/db/models"
import dbConnect from "@/lib/db/connect"

// Minimal script to add a dummy device to the current user
async function main() {
    // Mock session or find by email - since this is a script, we might need a hardcoded email or use first user
    // However, in Next.js scripts context, auth() might not work as expected without request context.
    // Instead, let's just find a user by email if we know it (or the first user).

    await dbConnect();

    const user = await User.findOne({}); // Get first user
    if (!user) {
        console.log("No users found.");
        return;
    }

    console.log(`Adding dummy device to user: ${user.email} `);

    const dummyDevice = {
        fingerprint: "dummy-fingerprint-" + Date.now(),
        name: "Manual Dummy Device",
        lastSeen: new Date()
    };

    await User.updateOne(
        { _id: user._id },
        { $push: { devices: dummyDevice } }
    );

    console.log("Device added successfully.");
}

// Since we cannot run this directly with simple `node` if it uses Next.js aliases (@/...), 
// we might need to rely on the user running this in a specific way or just instruct them.
// But we can create a temporary API route that does this securely if dev mode.
