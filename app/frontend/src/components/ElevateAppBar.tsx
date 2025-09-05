import { AppBar, Toolbar, useScrollTrigger } from '@mui/material';
import React from 'react';
import Logo from '../whitelabeling/Logo';
import SwitchThemeButton from './buttons/SwitchThemeButton';

interface Props {
  /**
   * Injected by the documentation to work in an iframe.
   * You won't need it on your project.
   */
  window?: () => Window;
  children?: React.ReactElement<{ elevation?: number }>;
}

function ElevationScroll(props: Props) {
  const { children, window } = props;
  // Note that you normally won't need to set the window ref as useScrollTrigger
  // will default to window.
  // This is only being set here because the demo is in an iframe.
  const trigger = useScrollTrigger({
    disableHysteresis: true,
    threshold: 0,
    target: window ? window() : undefined,
  });

  return children
    ? React.cloneElement(children, {
        elevation: trigger ? 4 : 0,
      })
    : null;
}

export default function ElevateAppBar(props: Props) {
  return (
    <>
      <ElevationScroll {...props}>
        <AppBar position="sticky" enableColorOnDark>
          <Toolbar>
            <Logo />
            <div style={{ flexGrow: 1 }} />
            <SwitchThemeButton />
          </Toolbar>
        </AppBar>
      </ElevationScroll>
    </>
  );
}
