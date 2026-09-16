import { Plus } from 'lucide-react'
import { useState } from 'react'
import type { Schemas } from '#/api/api.client'
import {
	Popover,
	PopoverContent,
	PopoverTrigger,
} from '#/components/ui/popover'
import { ConnectorSearch } from '#/pages/automation/ui/connector-search'

export interface AddNodeButtonProps {
	ariaLabel: string
	catalogue: Schemas.ConnectorDescriptorResponse[]
	onSelect: (connector: Schemas.ConnectorDescriptorResponse) => void
}

export function AddNodeButton({
	ariaLabel,
	catalogue,
	onSelect,
}: AddNodeButtonProps) {
	const [open, setOpen] = useState(false)

	return (
		<Popover open={open} onOpenChange={setOpen}>
			<PopoverTrigger asChild>
				<button
					type="button"
					aria-label={ariaLabel}
					onClick={(event) => event.stopPropagation()}
					className="nodrag flex size-5 items-center justify-center rounded-full border bg-background text-muted-foreground hover:bg-accent hover:text-accent-foreground"
				>
					<Plus className="size-3" />
				</button>
			</PopoverTrigger>
			<PopoverContent
				side="right"
				align="start"
				className="w-auto p-2"
				onClick={(event) => event.stopPropagation()}
			>
				<ConnectorSearch
					connectors={catalogue}
					onSelect={(connector) => {
						onSelect(connector)
						setOpen(false)
					}}
				/>
			</PopoverContent>
		</Popover>
	)
}
