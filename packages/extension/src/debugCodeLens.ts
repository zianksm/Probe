import * as parser from "@solidity-parser/parser";
import { ContractDefinition, FunctionDefinition } from "@solidity-parser/parser/dist/src/ast-types";
import * as vscode from 'vscode';


export const DebugCommand = "Probe.Debug" as const;

class DebugLens {
    public static inferDebugLens(text: string) {
        const source = parser.parse(text, { loc: true });

        return this.parseFunctions(source).map(this.toCodeLens);
    }

    private static toCodeLens(fn: FunctionDefinition): vscode.CodeLens {

        const loc = fn.loc!;

        const startPos = new vscode.Position(loc.start.line - 1, loc.start.column);
        const endPos = new vscode.Position(loc.end.line - 1, loc.end.column);

        const range = new vscode.Range(startPos, endPos);

        const cmd = {
            command: DebugCommand,
            title: "Debug",
            arguments: [fn]
        };

        return new vscode.CodeLens(range, cmd);
    }

    private static parseFunctions(source: unknown) {
        const functions: FunctionDefinition[] = [];
        parser.visit(source, {
            ContractDefinition: contract => {
                if (!this.isAbstract(contract)) { return; }

                parser.visit(contract, {
                    FunctionDefinition: fn => {
                        if (['external', 'public', 'default'].includes(fn.visibility) && fn.parameters.length == 0) {

                            functions.push(fn);
                        }

                    }
                });
            }
        });

        return functions;
    }

    private static isAbstract(contract: ContractDefinition): boolean {
        return contract.kind === 'contract';
    }
}


export class DebugLensAdapter implements vscode.CodeLensProvider {
    private _onDidChangeCodeLenses: vscode.EventEmitter<void> =
        new vscode.EventEmitter<void>();
    public readonly onDidChangeCodeLenses: vscode.Event<void> =
        this._onDidChangeCodeLenses.event;

    constructor() {
        vscode.workspace.onDidChangeConfiguration(() => {
            this._onDidChangeCodeLenses.fire();
        });
    }

    provideCodeLenses(document: vscode.TextDocument, token: vscode.CancellationToken): vscode.ProviderResult<vscode.CodeLens[]> {
        if (!document.fileName.includes(".t.sol")) return undefined;

        console.log("providing code lens");

        const res = DebugLens.inferDebugLens(document.getText());

        console.log(res);

        return res;


    }
    resolveCodeLens?(codeLens: vscode.CodeLens, token: vscode.CancellationToken): vscode.ProviderResult<vscode.CodeLens> {
        console.log("resolving code lens");

        console.log(codeLens);
        return codeLens;
    }

}