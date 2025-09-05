import './App.css';
import { Box, CssBaseline, Paper } from '@mui/material';

import { ThemeProvider, useColorScheme } from '@mui/material/styles';

import { theme } from '@whitelabeling/custom-theme';
import ElevateAppBar from '@components/ElevateAppBar';
import { HashRouter, Link, Navigate, Route, Routes } from 'react-router-dom';

import LoginPage from '@pages/LoginPage';
import OpenSource from '@pages/OpenSource';

function MyApp() {
  const { mode } = useColorScheme();
  if (!mode) return null;

  return (
    <>
      <CssBaseline />
      <HashRouter>
        <ElevateAppBar />
        <Paper elevation={0} sx={{ marginTop: 2, p: 2 }}>
          <Routes>
            <Route index element={<LoginPage />} />
            <Route path="open-source" element={<OpenSource />} />
            <Route path="*" element={<Navigate to="/" replace />} />
          </Routes>
        </Paper>
        <Box sx={{ p: 2 }}>
          <Link to="open-source">Open Source</Link>
        </Box>
      </HashRouter>
    </>
  );
}

export default function App() {
  return (
    <>
      <ThemeProvider theme={theme}>
        <MyApp />
      </ThemeProvider>
    </>
  );
}
