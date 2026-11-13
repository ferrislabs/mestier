import { Building2, GraduationCap } from 'lucide-react'
import type { ReactNode } from 'react'
import { cn } from '#/lib/utils'
import type { TutorKindChoice } from '#/pages/cursus/tutors/lib/tutors'

interface ChoiceCard {
	value: TutorKindChoice
	title: string
	description: string
	icon: ReactNode
}

const CHOICES: readonly ChoiceCard[] = [
	{
		value: 'company',
		title: 'Tuteur entreprise',
		description:
			"Salarié de l'entreprise d'accueil. Il reçoit une invitation pour se connecter.",
		icon: <Building2 aria-hidden className="size-5" />,
	},
	{
		value: 'school',
		title: 'Tuteur école',
		description: 'Un membre de votre équipe. Il utilise déjà Mestier.',
		icon: <GraduationCap aria-hidden className="size-5" />,
	},
]

export interface TutorKindChoiceProps {
	value: TutorKindChoice
	onChange: (value: TutorKindChoice) => void
}

export function TutorKindChoiceCards({
	value,
	onChange,
}: TutorKindChoiceProps) {
	return (
		<fieldset className="grid gap-3 sm:grid-cols-2">
			<legend className="sr-only">Type de tuteur</legend>
			{CHOICES.map((choice) => {
				const selected = choice.value === value
				return (
					<label
						key={choice.value}
						className={cn(
							'flex cursor-pointer flex-col gap-2 rounded-lg border bg-card p-4 transition has-focus-visible:ring-2 has-focus-visible:ring-ring',
							selected
								? 'border-primary ring-1 ring-primary'
								: 'hover:border-foreground/30',
						)}
					>
						<input
							type="radio"
							name="tutor-kind"
							value={choice.value}
							checked={selected}
							onChange={() => onChange(choice.value)}
							className="sr-only"
						/>
						<span className="text-muted-foreground">{choice.icon}</span>
						<span className="font-medium">{choice.title}</span>
						<span className="text-[13px] text-muted-foreground">
							{choice.description}
						</span>
					</label>
				)
			})}
		</fieldset>
	)
}
