import { API_URL } from "../main";
import { AuthStatus } from "../bindings/AuthStatus";

export async function checkAuthStatus(): Promise<AuthStatus> {
  try {
    const response = await fetch(`${API_URL}/auth/status`, {
      credentials: "include",
    });
    
    if (!response.ok) {
      throw new Error(`Auth status check failed: ${response.status}`);
    }
    
    return await response.json() as AuthStatus;
  } catch (error) {
    console.warn("Auth status check failed:", error);
    // If the auth status check fails, assume not authenticated
    return {
      authenticated: false,
      user_id: null,
      username: null,
    };
  }
}

export function redirectToAuth() {
  document.location.replace(`${API_URL}/auth/oidc`);
}

export async function authenticatedFetch(url: string, options: RequestInit = {}): Promise<Response> {
  const response = await fetch(url, {
    credentials: "include",
    redirect: "manual", // Don't follow redirects automatically
    ...options,
  });

  // If we get a redirect response (307, 302, etc.)
  if (response.status >= 300 && response.status < 400) {
    const location = response.headers.get("location");
    
    // If it's redirecting to auth, handle it
    if (location && location.includes("/auth/oidc")) {
      // Clear any stale cookies
      document.cookie = "USERID=; expires=Thu, 01 Jan 1970 00:00:00 UTC; path=/;";
      document.cookie = "USERNAME=; expires=Thu, 01 Jan 1970 00:00:00 UTC; path=/;";
      
      // Redirect to auth
      redirectToAuth();
      
      // Throw an error to prevent further processing
      throw new Error("Authentication required - redirecting to login");
    }
  }

  return response;
}