import DarkModeIcon from '@mui/icons-material/DarkMode';
import LightModeIcon from '@mui/icons-material/LightMode';
import { IconButton, useMediaQuery } from '@mui/material';
import { useColorScheme } from '@mui/material/styles';

export default function SwitchThemeButton() {
  const { mode, setMode } = useColorScheme();
  const prefersDarkMode = useMediaQuery('(prefers-color-scheme: dark)');

  return (
    <>
      {mode === 'dark' || (mode === 'system' && prefersDarkMode) ? (
        <IconButton
          aria-label="theme-switcher"
          sx={{ ml: 1, mr: 1 }}
          onClick={() => setMode('light')}
        >
          <LightModeIcon aria-label="dark-mode" />
        </IconButton>
      ) : (
        <IconButton
          aria-label="theme-switcher"
          sx={{ ml: 1, mr: 1 }}
          onClick={() => setMode('dark')}
        >
          <DarkModeIcon aria-label="light-mode" />
        </IconButton>
      )}
    </>
  );
}
