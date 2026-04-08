import { describe, it, expect } from 'vitest'
import { render, screen } from '@testing-library/react'

import { Home } from './home'

describe('Home', () => {
  it('should render the home screen correctly', () => {
    render(<Home />)

    expect(
      screen.getByRole('heading', { name: 'Welcome!', level: 2 })
    ).toBeInTheDocument()
  })
})
