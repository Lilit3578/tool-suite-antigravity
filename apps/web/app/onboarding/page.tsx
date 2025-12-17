
import { auth } from '@/auth';
import { User } from '@/lib/db/models';
import dbConnect from '@/lib/db/connect';
import { onboardUser } from '@/lib/actions/dashboard';
import { redirect } from 'next/navigation';

export default async function OnboardingPage() {
    const session = await auth();
    if (!session?.user?.email) {
        redirect('/login');
    }

    await dbConnect();
    const user = await User.findOne({ email: session.user.email }).lean();

    if (user?.name) {
        redirect('/dashboard');
    }

    return (
        <div className="flex min-h-screen flex-col items-center justify-center bg-gray-50 py-12 sm:px-6 lg:px-8">
            <div className="sm:mx-auto sm:w-full sm:max-w-md">
                <h2 className="mt-6 text-center text-3xl font-extrabold text-gray-900">
                    Welcome!
                </h2>
                <p className="mt-2 text-center text-sm text-gray-600">
                    Let&apos;s verify your details.
                </p>
            </div>

            <div className="mt-8 sm:mx-auto sm:w-full sm:max-w-md">
                <div className="bg-white py-8 px-4 shadow sm:rounded-lg sm:px-10">
                    <form action={onboardUser} className="space-y-6">
                        <div>
                            <label htmlFor="name" className="block text-sm font-medium text-gray-700">
                                What should we call you?
                            </label>
                            <div className="mt-1">
                                <input
                                    id="name"
                                    name="name"
                                    type="text"
                                    required
                                    className="appearance-none block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
                                    placeholder="Your Name"
                                />
                            </div>
                        </div>

                        <div>
                            <button
                                type="submit"
                                className="w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
                            >
                                Continue to Dashboard
                            </button>
                        </div>
                    </form>
                </div>
            </div>
        </div>
    );
}
