import { Braces } from 'lucide-react'
import { Button } from '#/components/ui/button'
import {
	Popover,
	PopoverContent,
	PopoverTrigger,
} from '#/components/ui/popover'
import {
	buildConnectorExpression,
	type ExamplePath,
} from '#/pages/automation/lib/expression'

export interface UpstreamConnectorOption {
	id: string
	/** The label the canvas shows for this node — never part of the
	 * expression itself (see `graph.ts`'s doc comment on `upstreamOf`: a
	 * rename must not change any expression). */
	label: string
	paths: ExamplePath[]
}

export interface ExpressionPickerProps {
	/** Already resolved to only the connectors upstream of the field being
	 * edited (`upstreamOf` + `flattenExamplePaths`, done by the feature
	 * layer) — a downstream or unreachable connector never appears here,
	 * because the backend would reject that expression at save time anyway. */
	upstream: UpstreamConnectorOption[]
	onInsert: (expression: string) => void
}

/** The picking list on its own, no trigger — `ExpressionPicker` below wraps
 * it in a popover for standalone use. The workflow editor's node panel
 * (`workflow-editor-ui.tsx`) renders this directly instead: `FieldForm`
 * already draws its own per-field "Insérer une expression" button, so
 * anchoring a second, separate popover trigger to it would either duplicate
 * that button or need a DOM ref `FieldForm` doesn't expose. Exported so
 * both call sites share one rendering, never two. */
export function ExpressionPickerList({
	upstream,
	onInsert,
}: ExpressionPickerProps) {
	if (upstream.length === 0) {
		return (
			<p className="px-2 py-3 text-sm text-muted-foreground">
				Aucun connecteur en amont — placez-en un avant celui-ci pour référencer
				sa sortie.
			</p>
		)
	}

	return (
		<div className="flex flex-col gap-3">
			{upstream.map((connector) => (
				<div key={connector.id} className="flex flex-col gap-1">
					<p className="px-2 text-xs font-semibold uppercase text-muted-foreground">
						{connector.label}
					</p>
					{connector.paths.map((example) => (
						<button
							key={example.path}
							type="button"
							className="flex items-center justify-between gap-2 rounded-md px-2 py-1.5 text-left text-sm hover:bg-muted"
							onClick={() =>
								onInsert(buildConnectorExpression(connector.id, example.path))
							}
						>
							<span className="truncate font-mono">
								{example.path === '' ? 'output' : example.path}
							</span>
							<span className="shrink-0 truncate text-xs text-muted-foreground">
								{example.preview}
							</span>
						</button>
					))}
				</div>
			))}
		</div>
	)
}

/** A field-agnostic trigger button + popover of insertable
 * `{{ connectors.<id>.output.<path> }}` expressions, for a context with no
 * existing "insert expression" button of its own to hook into. */
export function ExpressionPicker({
	upstream,
	onInsert,
}: ExpressionPickerProps) {
	return (
		<Popover>
			<PopoverTrigger asChild>
				<Button
					type="button"
					variant="ghost"
					size="icon-sm"
					title="Insérer une expression"
				>
					<Braces />
					<span className="sr-only">Insérer une expression</span>
				</Button>
			</PopoverTrigger>
			<PopoverContent className="max-h-80 w-80 overflow-y-auto p-2" align="end">
				<ExpressionPickerList upstream={upstream} onInsert={onInsert} />
			</PopoverContent>
		</Popover>
	)
}
