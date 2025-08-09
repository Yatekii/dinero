import { checkAuthStatus, redirectToAuth } from "./auth";

/**
 * Higher-order function that wraps any router loader with authentication check
 */
export function withAuth<T>(loader: () => Promise<T>): () => Promise<T> {
  return async () => {
    // Check authentication first
    const authStatus = await checkAuthStatus();
    
    if (!authStatus.authenticated) {
      // Redirect to auth and throw error to prevent further loading
      redirectToAuth();
      throw new Error("Authentication required");
    }
    
    // If authenticated, call the original loader
    return loader();
  };
}