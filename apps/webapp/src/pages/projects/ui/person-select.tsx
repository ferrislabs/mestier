import {
	Select,
	SelectContent,
	SelectItem,
	SelectTrigger,
	SelectValue,
} from '#/components/ui/select'
import { NO_PERSON } from '#/pages/projects/lib/form-model'
import type { Person } from '#/pages/projects/lib/model'

export function PersonSelect({
	id,
	value,
	people,
	onChange,
}: {
	id: string
	value: string
	people: readonly Person[]
	onChange: (value: string) => void
}) {
	return (
		<Select value={value} onValueChange={onChange}>
			<SelectTrigger id={id} className="w-full">
				<SelectValue />
			</SelectTrigger>
			<SelectContent>
				<SelectItem value={NO_PERSON}>Sans responsable</SelectItem>
				{people.map((person) => (
					<SelectItem key={person.id} value={person.id}>
						{person.name}
					</SelectItem>
				))}
			</SelectContent>
		</Select>
	)
}
