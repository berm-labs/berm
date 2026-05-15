import { Platform } from "react-native";

// Space Grotesk / Inter are loaded as custom fonts in production builds.
// Until the font assets are bundled we fall back to the platform system stack
// so text always renders. Display weights mirror the web masthead.
const systemMono = Platform.select({ ios: "Menlo", android: "monospace", default: "monospace" });
const systemSans = Platform.select({ ios: "System", android: "sans-serif", default: "System" });

export const typography = {
  display: {
    fontFamily: systemSans,
    fontWeight: "700" as const,
    letterSpacing: -0.5,
  },
  title: {
    fontFamily: systemSans,
    fontWeight: "600" as const,
  },
  body: {
    fontFamily: systemSans,
    fontWeight: "400" as const,
  },
  mono: {
    fontFamily: systemMono,
    fontWeight: "400" as const,
  },
};

export const spacing = {
  xs: 4,
  sm: 8,
  md: 16,
  lg: 24,
  xl: 32,
};
