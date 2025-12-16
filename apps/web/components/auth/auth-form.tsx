"use client"

import { useFormState, useFormStatus } from "react-dom"
import { login, signup } from "@/lib/actions/auth"
import Link from "next/link"

type AuthMode = "login" | "signup"

export function AuthForm({ mode }: { mode: AuthMode }) {
    const action = mode === "login" ? login : signup
    const [state, formAction] = useFormState(action, null)

    return (
        <div className="flex min-h-screen flex-col items-center justify-center bg-gray-50 p-4">
            <div className="w-full max-w-sm space-y-8 rounded-xl bg-white p-8 shadow-lg">
                <div className="text-center">
                    <h2 className="text-2xl font-bold tracking-tight text-gray-900">
                        {mode === "login" ? "Welcome back" : "Create an account"}
                    </h2>
                    <p className="mt-2 text-sm text-gray-600">
                        {mode === "login"
                            ? "Enter your email to sign in"
                            : "Enter your email to get started"}
                    </p>
                </div>

                <form action={formAction} className="space-y-6">
                    <div>
                        <label
                            htmlFor="email"
                            className="block text-sm font-medium text-gray-700"
                        >
                            Email address
                        </label>
                        <div className="mt-1">
                            <input
                                id="email"
                                name="email"
                                type="email"
                                autoComplete="email"
                                required
                                className="block w-full rounded-md border border-gray-300 px-3 py-2 shadow-sm focus:border-indigo-500 focus:outline-none focus:ring-indigo-500 sm:text-sm"
                                placeholder="you@example.com"
                            />
                        </div>
                    </div>

                    {state?.error && (
                        <div className="rounded-md bg-red-50 p-3 text-sm text-red-600">
                            {state.error}
                        </div>
                    )}

                    <SubmitButton mode={mode} />
                </form>

                <div className="text-center text-sm">
                    {mode === "login" ? (
                        <p>
                            Don&apos;t have an account?{" "}
                            <Link
                                href="/signup"
                                className="font-medium text-indigo-600 hover:text-indigo-500"
                            >
                                Sign up
                            </Link>
                        </p>
                    ) : (
                        <p>
                            Already have an account?{" "}
                            <Link
                                href="/login"
                                className="font-medium text-indigo-600 hover:text-indigo-500"
                            >
                                Log in
                            </Link>
                        </p>
                    )}
                </div>
            </div>
        </div>
    )
}

function SubmitButton({ mode }: { mode: AuthMode }) {
    const { pending } = useFormStatus()

    return (
        <button
            type="submit"
            disabled={pending}
            className="flex w-full justify-center rounded-md border border-transparent bg-indigo-600 px-4 py-2 text-sm font-medium text-white shadow-sm hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 disabled:opacity-50"
        >
            {pending
                ? "Sending Magic Link..."
                : mode === "login"
                    ? "Sign in with Email"
                    : "Sign up with Email"}
        </button>
    )
}
