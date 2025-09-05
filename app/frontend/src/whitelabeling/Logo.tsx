/*
 * Copyright (c) 2022 FLECS Technologies GmbH
 *
 * Created on Tue Sep 30 2022
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 * http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */
import React from 'react';
import { Box, IconButton, Typography } from '@mui/material';
import LogoSvg from '@assets/images/logo.svg';
import { Link } from 'react-router-dom';

const Logo: React.FC = () => {
  return (
    <>
      <Box
        component={Link}
        to="/"
        sx={{
          display: 'flex',
          alignItems: 'center',
          textDecoration: 'none',
          color: 'inherit',
        }}
      >
        <IconButton aria-label="logo" disabled={true}>
          <img src={LogoSvg} width="24" height="24" />
        </IconButton>
        <Typography variant="h6">FLECS</Typography>
      </Box>
    </>
  );
};

export default Logo;
