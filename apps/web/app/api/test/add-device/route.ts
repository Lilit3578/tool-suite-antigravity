
import { auth } from "@/auth";
import { User } from "@/lib/db/models";
import dbConnect from "@/lib/db/connect";
import { NextResponse } from "next/server";

export async function GET() {
    const session = await auth();

    // Security check: ensure only logged in users can do this (or remove if you want it open)
    if (!session?.user?.email) {
        return NextResponse.json({ error: "Unauthorized. Please log in first." }, { status: 401 });
    }

    await dbConnect();

    const dummyDevice = {
        fingerprint: "dummy-" + Math.random().toString(36).substring(7),
        name: "Test MacBook Pro " + Math.floor(Math.random() * 100),
        lastSeen: new Date()
    };

    const updatedUser = await User.findOneAndUpdate(
        { email: session.user.email },
        { $push: { devices: dummyDevice } },
        { new: true }
    );

    return NextResponse.json({
        success: true,
        message: "Dummy device added",
        devices: updatedUser.devices
    });
}
