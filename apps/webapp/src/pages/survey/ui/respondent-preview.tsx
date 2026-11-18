import { CircleCheck, Split } from 'lucide-react'
import type { ReactNode } from 'react'
import { NdvPanel } from '#/pages/automation/ui/ndv-panel'
import type { GraphNode } from '#/pages/survey/types'
import { QuestionStep } from '#/pages/survey-respond/ui/question-step'
import { RespondLayout } from '#/pages/survey-respond/ui/respond-layout'
import { RespondMessage } from '#/pages/survey-respond/ui/respond-message'

export interface BranchSummary {
	branch: string
	target: string | null
}

const IGNORE = () => undefined

function PhoneFrame({ children }: { children: ReactNode }) {
	return (
		<div className="mx-auto w-full max-w-[390px] overflow-hidden rounded-[2rem] border-8 border-foreground/85 bg-background shadow-xl">
			<div
				inert
				data-testid="respondent-preview"
				className="pointer-events-none max-h-[640px] min-h-[480px] select-none overflow-y-auto"
			>
				{children}
			</div>
		</div>
	)
}

export function RespondentPreview({
	node,
	surveyTitle,
	endMessage,
	sentence,
	branches,
}: {
	node: GraphNode
	surveyTitle: string
	endMessage: string
	sentence: string
	branches: readonly BranchSummary[]
}) {
	if (node.type === 'CONDITION') {
		return (
			<NdvPanel
				title="Aiguillage"
				caption="Invisible pour le répondant"
				tone="side"
			>
				<div className="grid gap-4">
					<p className="flex items-start gap-2 rounded-md border bg-card px-3 py-2 text-sm">
						<Split className="mt-0.5 size-4 shrink-0" aria-hidden />
						{sentence}
					</p>
					<ul aria-label="Branches de la condition" className="grid gap-2">
						{branches.map((item) => (
							<li
								key={item.branch}
								className="flex items-center justify-between gap-3 rounded-md border bg-card px-3 py-2 text-sm"
							>
								<span className="font-medium">{item.branch}</span>
								<span
									className={
										item.target ? 'truncate' : 'text-muted-foreground italic'
									}
								>
									{item.target ?? 'Sortie non reliée'}
								</span>
							</li>
						))}
					</ul>
				</div>
			</NdvPanel>
		)
	}
	return (
		<NdvPanel title="Aperçu répondant" caption="Non interactif" tone="side">
			<PhoneFrame>
				<RespondLayout variant="member" title={surveyTitle}>
					{node.type === 'QUESTION' ? (
						<QuestionStep
							question={{
								...node,
								label:
									node.label.trim() === ''
										? 'Votre question apparaîtra ici'
										: node.label,
							}}
							draft={undefined}
							photos={[]}
							photoNotice={null}
							progress={null}
							error={null}
							headingRef={{ current: null }}
							onChange={IGNORE}
							onSubmit={IGNORE}
							onAddPhotos={IGNORE}
							onRemovePhoto={IGNORE}
						/>
					) : (
						<RespondMessage
							icon={CircleCheck}
							tone="success"
							title="Réponse envoyée"
							message={endMessage}
						/>
					)}
				</RespondLayout>
			</PhoneFrame>
		</NdvPanel>
	)
}
