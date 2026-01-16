package com.pdfoxide.internal;

import java.lang.ref.Cleaner;

/**
 * Internal wrapper for native Rust pointers with automatic cleanup.
 *
 * <p>Uses Java 9+ Cleaner API for guaranteed resource cleanup even if
 * close() is not called explicitly. This prevents memory leaks by registering
 * a cleanup action that runs when the NativeHandle is garbage collected.
 *
 * <p>Thread-safe for reading the pointer, but not for concurrent close() calls.
 *
 * @since 1.0.0
 */
public final class NativeHandle implements AutoCloseable {
    private static final Cleaner CLEANER = Cleaner.create();

    private final long ptr;
    private final Cleaner.Cleanable cleanable;
    private volatile boolean closed = false;

    /**
     * Functional interface for native cleanup operations.
     */
    @FunctionalInterface
    public interface NativeFinalizer {
        /**
         * Called to finalize the native resource.
         *
         * @param ptr the native pointer to clean up
         */
        void finalize(long ptr);
    }

    /**
     * Creates a new NativeHandle that wraps a native Rust pointer.
     *
     * <p>The provided finalizer will be called when:
     * <ul>
     *   <li>close() is called explicitly, or
     *   <li>The NativeHandle is garbage collected (via Cleaner)
     * </ul>
     *
     * @param ptr the native pointer from Rust (must not be 0/null)
     * @param finalizer cleanup function to call on finalization
     * @throws IllegalArgumentException if ptr is 0
     */
    public NativeHandle(long ptr, NativeFinalizer finalizer) {
        if (ptr == 0) {
            throw new IllegalArgumentException("Native pointer cannot be null (0)");
        }
        this.ptr = ptr;
        this.cleanable = CLEANER.register(this, () -> finalizer.finalize(ptr));
    }

    /**
     * Gets the native pointer value.
     *
     * @return the native pointer
     * @throws IllegalStateException if the handle has been closed
     */
    public long ptr() {
        if (closed) {
            throw new IllegalStateException("Native handle has been closed");
        }
        return ptr;
    }

    /**
     * Checks if the handle is closed.
     *
     * @return true if closed, false otherwise
     */
    public boolean isClosed() {
        return closed;
    }

    /**
     * Closes and cleans up the native resource.
     *
     * <p>Safe to call multiple times - subsequent calls are no-ops.
     * Calling this method ensures immediate cleanup rather than waiting
     * for garbage collection.
     */
    @Override
    public void close() {
        if (!closed) {
            closed = true;
            cleanable.clean();
        }
    }
}
