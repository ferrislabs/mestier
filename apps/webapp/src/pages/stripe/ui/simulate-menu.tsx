import { ChevronDown, FlaskConical } from 'lucide-react'
import { Button } from '#/components/ui/button'
import {
	DropdownMenu,
	DropdownMenuContent,
	DropdownMenuItem,
	DropdownMenuLabel,
	DropdownMenuSeparator,
	DropdownMenuTrigger,
} from '#/components/ui/dropdown-menu'
import { SIMULATION_SCENARIOS } from '#/pages/stripe/lib/stripe-model'
import type { StripeScenario } from '#/pages/stripe/types'

export function SimulateMenu({
	onSimulate,
}: {
	onSimulate: (scenario: StripeScenario) => void
}) {
	return (
		<DropdownMenu>
			<DropdownMenuTrigger asChild>
				<Button type="button" variant="outline">
					<FlaskConical aria-hidden />
					Simuler un webhook
					<ChevronDown aria-hidden />
				</Button>
			</DropdownMenuTrigger>
			<DropdownMenuContent align="end" className="w-72">
				<DropdownMenuLabel>Scénario à rejouer</DropdownMenuLabel>
				<DropdownMenuSeparator />
				{SIMULATION_SCENARIOS.map((scenario) => (
					<DropdownMenuItem
						key={scenario.id}
						className="flex-col items-start gap-0.5"
						onSelect={() => onSimulate(scenario.id)}
					>
						<span className="font-medium">{scenario.label}</span>
						<span className="text-muted-foreground text-xs">
							{scenario.hint}
						</span>
					</DropdownMenuItem>
				))}
			</DropdownMenuContent>
		</DropdownMenu>
	)
}
