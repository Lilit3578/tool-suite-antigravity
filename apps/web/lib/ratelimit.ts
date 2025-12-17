import { Ratelimit } from "@upstash/ratelimit";
import { Redis } from "@upstash/redis";
import { NextRequest } from "next/server";

/**
 * Singleton Redis client for Upstash
 * Uses REST API for serverless compatibility
 */
const redis = new Redis({
    url: process.env.UPSTASH_REDIS_REST_URL!,
    token: process.env.UPSTASH_REDIS_REST_TOKEN!,
});

/**
 * Rate limiter with sliding window algorithm
 * Limit: 10 requests per 10 seconds
 * 
 * Sliding window prevents "boundary burst" attacks where users
 * could send 10 requests at 00:00:59 and 10 more at 00:01:00
 */
export const ratelimit = new Ratelimit({
    redis,
    limiter: Ratelimit.slidingWindow(10, "10 s"),
    analytics: true, // Enable analytics for Upstash Dashboard monitoring
    prefix: "@upstash/ratelimit",
});

/**
 * Extract client IP from request headers
 * Priority: x-forwarded-for > x-real-ip > fallback
 * 
 * @param req - Next.js request object
 * @returns Client IP address or fallback identifier
 */
export function getClientIp(req: NextRequest): string {
    const forwarded = req.headers.get("x-forwarded-for");
    if (forwarded) {
        // x-forwarded-for can contain multiple IPs, take the first one
        return forwarded.split(",")[0].trim();
    }

    const realIp = req.headers.get("x-real-ip");
    if (realIp) {
        return realIp;
    }

    // Fallback for local development or when headers are not available
    return "anonymous";
}
