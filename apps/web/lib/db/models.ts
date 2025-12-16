import { Schema, model, models, InferSchemaType } from 'mongoose';

const DeviceSchema = new Schema({
    fingerprint: { type: String, required: true },
    name: { type: String, required: true },
    lastSeen: { type: Date, default: Date.now }
});

const UserSchema = new Schema({
    email: { type: String, required: true, unique: true, index: true },
    plan: { type: String, enum: ['free', 'paid'], default: 'free' },
    stripeCustomerId: { type: String, unique: true, sparse: true },
    devices: {
        type: [DeviceSchema],
        validate: [
            // eslint-disable-next-line @typescript-eslint/no-explicit-any
            function (val: any[]) {
                return val.length <= 3;
            },
            '{PATH} exceeds the limit of 3'
        ]
    }
});

const UsageLogSchema = new Schema({
    userId: { type: Schema.Types.ObjectId, ref: 'User', required: true },
    date: { type: String, required: true }, // Format: "YYYY-MM-DD"
    counts: { type: Map, of: Number }
});

// Compound Unique Index: One log per user per day
UsageLogSchema.index({ userId: 1, date: 1 }, { unique: true });

// Export Models
export const User = models.User || model('User', UserSchema);
export const UsageLog = models.UsageLog || model('UsageLog', UsageLogSchema);

// Export Inferred Types
export type UserType = InferSchemaType<typeof UserSchema>;
export type UsageLogType = InferSchemaType<typeof UsageLogSchema>;
