import { MantineThemeOverride } from '@mantine/core';

export const theme: MantineThemeOverride = {
  primaryColor: 'pink',
  
  colors: {
    pink: [
      '#FFE4F1', // 0 - lightest
      '#FFD1E3', // 1
      '#FFB6D9', // 2
      '#FF9ACF', // 3
      '#FF7EC5', // 4
      '#FF1493', // 5 - primary
      '#E6127F', // 6
      '#CC106C', // 7
      '#B30E59', // 8
      '#990C46', // 9 - darkest
    ],
    // Dark mode specific pinks
    darkPink: [
      '#FFF0F8',
      '#FFE0EF',
      '#FFD1E6',
      '#FFC2DD',
      '#FFB3D4',
      '#DB7093', // 5 - primary for dark mode
      '#C71585', // 6
      '#B3135E',
      '#9F1157',
      '#8B0F50',
    ],
  },

  primaryShade: { light: 5, dark: 6 },

  fontFamily: 'Inter, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif',
  fontFamilyMonospace: '"Fira Code", "Courier New", Courier, monospace',
  
  headings: {
    fontFamily: 'Inter, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
    fontWeight: '700',
    sizes: {
      h1: { fontSize: '2.125rem', lineHeight: '1.3' },
      h2: { fontSize: '1.625rem', lineHeight: '1.35' },
      h3: { fontSize: '1.375rem', lineHeight: '1.4' },
      h4: { fontSize: '1.125rem', lineHeight: '1.45' },
      h5: { fontSize: '1rem', lineHeight: '1.5' },
      h6: { fontSize: '0.875rem', lineHeight: '1.5' },
    },
  },

  shadows: {
    xs: '0 1px 3px rgba(255, 20, 147, 0.05), 0 1px 2px rgba(255, 20, 147, 0.1)',
    sm: '0 1px 3px rgba(255, 20, 147, 0.1), 0 4px 6px rgba(255, 20, 147, 0.1)',
    md: '0 4px 6px rgba(255, 20, 147, 0.07), 0 10px 15px rgba(255, 20, 147, 0.1)',
    lg: '0 10px 15px rgba(255, 20, 147, 0.1), 0 20px 25px rgba(255, 20, 147, 0.1)',
    xl: '0 20px 25px rgba(255, 20, 147, 0.1), 0 25px 50px rgba(255, 20, 147, 0.15)',
    // Custom pink glow shadows
    pinkGlow: '0 0 20px rgba(255, 20, 147, 0.3), 0 0 40px rgba(255, 20, 147, 0.2)',
    pinkGlowStrong: '0 0 30px rgba(255, 20, 147, 0.5), 0 0 60px rgba(255, 20, 147, 0.3)',
  },

  radius: {
    xs: '0.25rem',
    sm: '0.375rem',
    md: '0.5rem',
    lg: '0.75rem',
    xl: '1rem',
  },

  spacing: {
    xs: '0.625rem',
    sm: '0.75rem',
    md: '1rem',
    lg: '1.25rem',
    xl: '1.5rem',
  },

  breakpoints: {
    xs: '36em',
    sm: '48em',
    md: '62em',
    lg: '75em',
    xl: '88em',
  },

  defaultGradient: {
    from: '#FF1493',
    to: '#C71585',
    deg: 135,
  },

  // Custom gradients
  other: {
    gradients: {
      pinkPurple: 'linear-gradient(135deg, #FF1493 0%, #C71585 50%, #9b59b6 100%)',
      pinkBlue: 'linear-gradient(135deg, #FF1493 0%, #C71585 50%, #4A90E2 100%)',
      pinkSunset: 'linear-gradient(135deg, #FFB6C1 0%, #FF1493 50%, #C71585 100%)',
      darkPink: 'linear-gradient(135deg, #C71585 0%, #990C46 100%)',
      glowPink: 'radial-gradient(circle, rgba(255, 20, 147, 0.3) 0%, transparent 70%)',
    },
    transitions: {
      default: 'all 300ms cubic-bezier(0.4, 0, 0.2, 1)',
      fast: 'all 150ms cubic-bezier(0.4, 0, 0.2, 1)',
      slow: 'all 500ms cubic-bezier(0.4, 0, 0.2, 1)',
    },
  },

  components: {
    Button: {
      styles: (theme: any) => ({
        root: {
          transition: 'all 300ms cubic-bezier(0.4, 0, 0.2, 1)',
          '&:hover': {
            transform: 'translateY(-2px)',
            boxShadow: theme.shadows.pinkGlow,
          },
          '&:active': {
            transform: 'translateY(0)',
          },
        },
      }),
    },
    Card: {
      styles: (theme: any) => ({
        root: {
          transition: 'all 300ms cubic-bezier(0.4, 0, 0.2, 1)',
          borderColor: theme.colorScheme === 'dark' ? '#C71585' : '#FFB6C1',
          '&:hover': {
            borderColor: '#FF1493',
            boxShadow: theme.shadows.pinkGlow,
          },
        },
      }),
    },
    Input: {
      styles: (theme: any) => ({
        input: {
          transition: 'all 300ms cubic-bezier(0.4, 0, 0.2, 1)',
          '&:focus': {
            borderColor: '#FF1493',
            boxShadow: '0 0 0 3px rgba(255, 20, 147, 0.1)',
          },
        },
      }),
    },
    Modal: {
      styles: {
        modal: {
          backdropFilter: 'blur(10px)',
        },
      },
    },
  },
};

export default theme;
