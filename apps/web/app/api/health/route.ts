import { NextResponse } from 'next/server';
import dbConnect from '@/lib/db/connect';

export async function GET() {
    try {
        await dbConnect();
        return NextResponse.json({ status: "Database Connected" }, { status: 200 });
    } catch (error) {
        console.error("Database connection error:", error);
        return NextResponse.json({ error: "Database Connection Failed" }, { status: 500 });
    }
}
