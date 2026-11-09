import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { ReadOnlyNoticeUI, VoiceChannelUI } from './channel-notice-ui'

describe('VoiceChannelUI', () => {
	it('says voice is coming and that no text is accepted', () => {
		render(<VoiceChannelUI />)

		expect(screen.getByText('Salon vocal')).toBeDefined()
		expect(screen.getByText(/n’accepte pas de messages texte/)).toBeDefined()
	})
})

describe('ReadOnlyNoticeUI', () => {
	it('replaces the composer with the reason', () => {
		render(<ReadOnlyNoticeUI label="Lecture seule" />)

		expect(screen.getByText('Lecture seule')).toBeDefined()
	})
})
