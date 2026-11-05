import { Check, ChevronsUpDown } from 'lucide-react'
import { useState } from 'react'
import { Button } from '#/components/ui/button'
import {
	Command,
	CommandEmpty,
	CommandGroup,
	CommandInput,
	CommandItem,
	CommandList,
} from '#/components/ui/command'
import {
	Popover,
	PopoverContent,
	PopoverTrigger,
} from '#/components/ui/popover'
import type { Customer } from '#/hooks/use-customers'
import { cn } from '#/lib/utils'
import { ACCOUNT_STATUS_LABELS, accountStatusOf } from '#/pages/customers/types'
import { hueOf, NameAvatar } from '#/pages/customers/ui/avatars'

export function CustomerPicker({
	id,
	customers,
	value,
	onChange,
	placeholder = 'Choisir un client',
	disabled,
	invalid,
	className,
}: {
	id?: string
	customers: readonly Customer[]
	value: string | null
	onChange: (customerId: string) => void
	placeholder?: string
	disabled?: boolean
	invalid?: boolean
	className?: string
}) {
	const [open, setOpen] = useState(false)
	const selected = customers.find((customer) => customer.id === value)
	return (
		<Popover open={open} onOpenChange={setOpen}>
			<PopoverTrigger asChild>
				<Button
					id={id}
					type="button"
					variant="outline"
					role="combobox"
					aria-expanded={open}
					aria-invalid={invalid || undefined}
					disabled={disabled}
					className={cn('w-full justify-between font-normal', className)}
				>
					{selected ? (
						<span className="flex min-w-0 items-center gap-2">
							<NameAvatar
								name={selected.name}
								hue={hueOf(selected.id)}
								size="sm"
							/>
							<span className="truncate">{selected.name}</span>
						</span>
					) : (
						<span className="text-muted-foreground">{placeholder}</span>
					)}
					<ChevronsUpDown className="size-4 opacity-50" aria-hidden />
				</Button>
			</PopoverTrigger>
			<PopoverContent
				align="start"
				className="w-[--radix-popover-trigger-width] min-w-72 p-0"
			>
				<Command>
					<CommandInput placeholder="Rechercher un client…" />
					<CommandList>
						<CommandEmpty>Aucun client trouvé.</CommandEmpty>
						<CommandGroup>
							{customers.map((customer) => (
								<CommandItem
									key={customer.id}
									value={`${customer.name} ${customer.email ?? ''}`}
									onSelect={() => {
										onChange(customer.id)
										setOpen(false)
									}}
								>
									<NameAvatar
										name={customer.name}
										hue={hueOf(customer.id)}
										size="sm"
									/>
									<span className="min-w-0 flex-1 truncate">
										{customer.name}
									</span>
									<span className="text-muted-foreground text-xs">
										{ACCOUNT_STATUS_LABELS[accountStatusOf(customer.status)]}
									</span>
									<Check
										className={cn(
											'size-4',
											customer.id === value ? 'opacity-100' : 'opacity-0',
										)}
										aria-hidden
									/>
								</CommandItem>
							))}
						</CommandGroup>
					</CommandList>
				</Command>
			</PopoverContent>
		</Popover>
	)
}
