import {
  abstractSyntaxTree,
  findClosingBrace,
  hasPipeOperator,
} from './abstractSyntaxTree'
import { lexer } from './tokeniser'

describe('findClosingBrace', () => {
  test('finds the closing brace', () => {
    const basic = '( hey )'
    expect(findClosingBrace(lexer(basic), 0)).toBe(4)

    const handlesNonZeroIndex =
      '(indexForBracketToRightOfThisIsTwo(shouldBeFour)AndNotThisSix)'
    expect(findClosingBrace(lexer(handlesNonZeroIndex), 2)).toBe(4)
    expect(findClosingBrace(lexer(handlesNonZeroIndex), 0)).toBe(6)

    const handlesNested =
      '{a{b{c(}d]}eathou athoeu tah u} thatOneToTheLeftIsLast }'
    expect(findClosingBrace(lexer(handlesNested), 0)).toBe(18)

    // throws when not started on a brace
    expect(() => findClosingBrace(lexer(handlesNested), 1)).toThrow()
  })
})

describe('testing AST', () => {
  test('test 5 + 6', () => {
    const tokens = lexer('5 +6')
    const result = abstractSyntaxTree(tokens)
    expect(result).toEqual({
      type: 'Program',
      start: 0,
      end: 4,
      body: [
        {
          type: 'ExpressionStatement',
          start: 0,
          end: 4,
          expression: {
            type: 'BinaryExpression',
            start: 0,
            end: 4,
            left: {
              type: 'Literal',
              start: 0,
              end: 1,
              value: 5,
              raw: '5',
            },
            operator: '+',
            right: {
              type: 'Literal',
              start: 3,
              end: 4,
              value: 6,
              raw: '6',
            },
          },
        },
      ],
    })
  })
  test('test const myVar = 5', () => {
    const tokens = lexer('const myVar = 5')
    const { body } = abstractSyntaxTree(tokens)
    expect(body).toEqual([
      {
        type: 'VariableDeclaration',
        start: 0,
        end: 15,
        kind: 'const',
        declarations: [
          {
            type: 'VariableDeclarator',
            start: 6,
            end: 15,
            id: {
              type: 'Identifier',
              start: 6,
              end: 11,
              name: 'myVar',
            },
            init: {
              type: 'Literal',
              start: 14,
              end: 15,
              value: 5,
              raw: '5',
            },
          },
        ],
      },
    ])
  })
  test('test multi-line', () => {
    const code = `const myVar = 5
const newVar = myVar + 1
`
    const tokens = lexer(code)
    const { body } = abstractSyntaxTree(tokens)
    expect(body).toEqual([
      {
        type: 'VariableDeclaration',
        start: 0,
        end: 15,
        kind: 'const',
        declarations: [
          {
            type: 'VariableDeclarator',
            start: 6,
            end: 15,
            id: {
              type: 'Identifier',
              start: 6,
              end: 11,
              name: 'myVar',
            },
            init: {
              type: 'Literal',
              start: 14,
              end: 15,
              value: 5,
              raw: '5',
            },
          },
        ],
      },
      {
        type: 'VariableDeclaration',
        start: 16,
        end: 40,
        kind: 'const',
        declarations: [
          {
            type: 'VariableDeclarator',
            start: 22,
            end: 40,
            id: {
              type: 'Identifier',
              start: 22,
              end: 28,
              name: 'newVar',
            },
            init: {
              type: 'BinaryExpression',
              start: 31,
              end: 40,
              left: {
                type: 'Identifier',
                start: 31,
                end: 36,
                name: 'myVar',
              },
              operator: '+',
              right: {
                type: 'Literal',
                start: 39,
                end: 40,
                value: 1,
                raw: '1',
              },
            },
          },
        ],
      },
    ])
  })
  test('test using std function "log"', () => {
    const code = `log(5, "hello", aIdentifier)`
    const tokens = lexer(code)
    const { body } = abstractSyntaxTree(tokens)
    expect(body).toEqual([
      {
        type: 'ExpressionStatement',
        start: 0,
        end: 28,
        expression: {
          type: 'CallExpression',
          start: 0,
          end: 28,
          callee: {
            type: 'Identifier',
            start: 0,
            end: 3,
            name: 'log',
          },
          arguments: [
            {
              type: 'Literal',
              start: 4,
              end: 5,
              value: 5,
              raw: '5',
            },
            {
              type: 'Literal',
              start: 7,
              end: 14,
              value: 'hello',
              raw: '"hello"',
            },
            {
              type: 'Identifier',
              start: 16,
              end: 27,
              name: 'aIdentifier',
            },
          ],
          optional: false,
        },
      },
    ])
  })
})

describe('testing function declaration', () => {
  test('fn funcN = () => {}', () => {
    const tokens = lexer('fn funcN = () => {}')
    const { body } = abstractSyntaxTree(tokens)
    expect(body).toEqual([
      {
        type: 'VariableDeclaration',
        start: 0,
        end: 19,
        kind: 'fn',
        declarations: [
          {
            type: 'VariableDeclarator',
            start: 3,
            end: 19,
            id: {
              type: 'Identifier',
              start: 3,
              end: 8,
              name: 'funcN',
            },
            init: {
              type: 'FunctionExpression',
              start: 11,
              end: 19,
              id: null,
              params: [],
              body: {
                type: 'BlockStatement',
                start: 17,
                end: 19,
                body: [],
              },
            },
          },
        ],
      },
    ])
  })
  test('fn funcN = (a, b) => {return a + b}', () => {
    const tokens = lexer(
      ['fn funcN = (a, b) => {', '  return a + b', '}'].join('\n')
    )
    const { body } = abstractSyntaxTree(tokens)
    expect(body).toEqual([
      {
        type: 'VariableDeclaration',
        start: 0,
        end: 39,
        kind: 'fn',
        declarations: [
          {
            type: 'VariableDeclarator',
            start: 3,
            end: 39,
            id: {
              type: 'Identifier',
              start: 3,
              end: 8,
              name: 'funcN',
            },
            init: {
              type: 'FunctionExpression',
              start: 11,
              end: 39,
              id: null,
              params: [
                {
                  type: 'Identifier',
                  start: 12,
                  end: 13,
                  name: 'a',
                },
                {
                  type: 'Identifier',
                  start: 15,
                  end: 16,
                  name: 'b',
                },
              ],
              body: {
                type: 'BlockStatement',
                start: 21,
                end: 39,
                body: [
                  {
                    type: 'ReturnStatement',
                    start: 25,
                    end: 37,
                    argument: {
                      type: 'BinaryExpression',
                      start: 32,
                      end: 37,
                      left: {
                        type: 'Identifier',
                        start: 32,
                        end: 33,
                        name: 'a',
                      },
                      operator: '+',
                      right: {
                        type: 'Identifier',
                        start: 36,
                        end: 37,
                        name: 'b',
                      },
                    },
                  },
                ],
              },
            },
          },
        ],
      },
    ])
  })
  test('call expression assignment', () => {
    const tokens = lexer(
      `fn funcN = (a, b) => { return a + b }
const myVar = funcN(1, 2)`
    )
    const { body } = abstractSyntaxTree(tokens)
    expect(body).toEqual([
      {
        type: 'VariableDeclaration',
        start: 0,
        end: 37,
        kind: 'fn',
        declarations: [
          {
            type: 'VariableDeclarator',
            start: 3,
            end: 37,
            id: {
              type: 'Identifier',
              start: 3,
              end: 8,
              name: 'funcN',
            },
            init: {
              type: 'FunctionExpression',
              start: 11,
              end: 37,
              id: null,
              params: [
                {
                  type: 'Identifier',
                  start: 12,
                  end: 13,
                  name: 'a',
                },
                {
                  type: 'Identifier',
                  start: 15,
                  end: 16,
                  name: 'b',
                },
              ],
              body: {
                type: 'BlockStatement',
                start: 21,
                end: 37,
                body: [
                  {
                    type: 'ReturnStatement',
                    start: 23,
                    end: 35,
                    argument: {
                      type: 'BinaryExpression',
                      start: 30,
                      end: 35,
                      left: {
                        type: 'Identifier',
                        start: 30,
                        end: 31,
                        name: 'a',
                      },
                      operator: '+',
                      right: {
                        type: 'Identifier',
                        start: 34,
                        end: 35,
                        name: 'b',
                      },
                    },
                  },
                ],
              },
            },
          },
        ],
      },
      {
        type: 'VariableDeclaration',
        start: 38,
        end: 63,
        kind: 'const',
        declarations: [
          {
            type: 'VariableDeclarator',
            start: 44,
            end: 63,
            id: {
              type: 'Identifier',
              start: 44,
              end: 49,
              name: 'myVar',
            },
            init: {
              type: 'CallExpression',
              start: 52,
              end: 63,
              callee: {
                type: 'Identifier',
                start: 52,
                end: 57,
                name: 'funcN',
              },
              arguments: [
                {
                  type: 'Literal',
                  start: 58,
                  end: 59,
                  value: 1,
                  raw: '1',
                },
                {
                  type: 'Literal',
                  start: 61,
                  end: 62,
                  value: 2,
                  raw: '2',
                },
              ],
              optional: false,
            },
          },
        ],
      },
    ])
  })
})

describe('structures specific to this lang', () => {
  test('sketch', () => {
    let code = `sketch mySketch {
  path myPath = lineTo(0,1)
  lineTo(1,1)
  path rightPath = lineTo(1,0)
  close()
}
`
    const tokens = lexer(code)
    const { body } = abstractSyntaxTree(tokens)
    expect(body).toEqual([
      {
        type: 'VariableDeclaration',
        start: 0,
        end: 102,
        kind: 'sketch',
        declarations: [
          {
            type: 'VariableDeclarator',
            start: 7,
            end: 102,
            id: {
              type: 'Identifier',
              start: 7,
              end: 15,
              name: 'mySketch',
            },
            init: {
              type: 'SketchExpression',
              start: 16,
              end: 102,
              body: {
                type: 'BlockStatement',
                start: 16,
                end: 102,
                body: [
                  {
                    type: 'VariableDeclaration',
                    start: 20,
                    end: 45,
                    kind: 'path',
                    declarations: [
                      {
                        type: 'VariableDeclarator',
                        start: 25,
                        end: 45,
                        id: {
                          type: 'Identifier',
                          start: 25,
                          end: 31,
                          name: 'myPath',
                        },
                        init: {
                          type: 'CallExpression',
                          start: 34,
                          end: 45,
                          callee: {
                            type: 'Identifier',
                            start: 34,
                            end: 40,
                            name: 'lineTo',
                          },
                          arguments: [
                            {
                              type: 'Literal',
                              start: 41,
                              end: 42,
                              value: 0,
                              raw: '0',
                            },
                            {
                              type: 'Literal',
                              start: 43,
                              end: 44,
                              value: 1,
                              raw: '1',
                            },
                          ],
                          optional: false,
                        },
                      },
                    ],
                  },
                  {
                    type: 'ExpressionStatement',
                    start: 48,
                    end: 59,
                    expression: {
                      type: 'CallExpression',
                      start: 48,
                      end: 59,
                      callee: {
                        type: 'Identifier',
                        start: 48,
                        end: 54,
                        name: 'lineTo',
                      },
                      arguments: [
                        {
                          type: 'Literal',
                          start: 55,
                          end: 56,
                          value: 1,
                          raw: '1',
                        },
                        {
                          type: 'Literal',
                          start: 57,
                          end: 58,
                          value: 1,
                          raw: '1',
                        },
                      ],
                      optional: false,
                    },
                  },
                  {
                    type: 'VariableDeclaration',
                    start: 62,
                    end: 90,
                    kind: 'path',
                    declarations: [
                      {
                        type: 'VariableDeclarator',
                        start: 67,
                        end: 90,
                        id: {
                          type: 'Identifier',
                          start: 67,
                          end: 76,
                          name: 'rightPath',
                        },
                        init: {
                          type: 'CallExpression',
                          start: 79,
                          end: 90,
                          callee: {
                            type: 'Identifier',
                            start: 79,
                            end: 85,
                            name: 'lineTo',
                          },
                          arguments: [
                            {
                              type: 'Literal',
                              start: 86,
                              end: 87,
                              value: 1,
                              raw: '1',
                            },
                            {
                              type: 'Literal',
                              start: 88,
                              end: 89,
                              value: 0,
                              raw: '0',
                            },
                          ],
                          optional: false,
                        },
                      },
                    ],
                  },
                  {
                    type: 'ExpressionStatement',
                    start: 93,
                    end: 100,
                    expression: {
                      type: 'CallExpression',
                      start: 93,
                      end: 100,
                      callee: {
                        type: 'Identifier',
                        start: 93,
                        end: 98,
                        name: 'close',
                      },
                      arguments: [],
                      optional: false,
                    },
                  },
                ],
              },
            },
          },
        ],
      },
    ])
  })
})
describe('testing hasPipeOperator', () => {
  test('hasPipeOperator is true', () => {
    let code = `sketch mySketch {
  lineTo(2, 3)
} |> rx(45, %)
`

    const tokens = lexer(code)
    expect(hasPipeOperator(tokens, 0)).toEqual({
      index: 16,
      token: { end: 37, start: 35, type: 'operator', value: '|>' },
    })
  })
  test('matches the first pipe', () => {
    let code = `sketch mySketch {
  lineTo(2, 3)
} |> rx(45, %) |> rx(45, %)
`
    const tokens = lexer(code)
    const result = hasPipeOperator(tokens, 0)
    expect(result).toEqual({
      index: 16,
      token: { end: 37, start: 35, type: 'operator', value: '|>' },
    })
    if (!result) throw new Error('should not happen')
    expect(code.slice(result.token.start, result.token.end)).toEqual('|>')
  })
  test('hasPipeOperator is false when the pipe operator is after a new variable declaration', () => {
    let code = `sketch mySketch {
  lineTo(2, 3)
}
const yo = myFunc(9()
  |> rx(45, %)
`
    const tokens = lexer(code)
    expect(hasPipeOperator(tokens, 0)).toEqual(false)
  })
  test('hasPipeOperator with binary expression', () => {
    let code = `const myVar2 = 5 + 1 |> myFn(%)`
    const tokens = lexer(code)
    const result = hasPipeOperator(tokens, 1)
    expect(result).toEqual({
      index: 12,
      token: { end: 23, start: 21, type: 'operator', value: '|>' },
    })
    if (!result) throw new Error('should not happen')
    expect(code.slice(result.token.start, result.token.end)).toEqual('|>')
  })
  test('hasPipeOperator of called mid sketchExpression on a callExpression, and called at the start of the sketchExpression at "{"', () => {
    const code = [
      'sketch mySk1 {',
      '  lineTo(1,1)',
      '  path myPath = lineTo(0, 1)',
      '  lineTo(1,1)',
      '} |> rx(90, %)',
      'show(mySk1)',
    ].join('\n')
    const tokens = lexer(code)
    const tokenWithMyPathIndex = tokens.findIndex(
      ({ value }) => value === 'myPath'
    )
    const tokenWithLineToIndexForVarDecIndex = tokens.findIndex(
      ({ value }, index) => value === 'lineTo' && index > tokenWithMyPathIndex
    )
    const result = hasPipeOperator(tokens, tokenWithLineToIndexForVarDecIndex)
    expect(result).toBe(false)

    const braceTokenIndex = tokens.findIndex(({ value }) => value === '{')
    const result2 = hasPipeOperator(tokens, braceTokenIndex)
    expect(result2).toEqual({
      index: 36,
      token: { end: 76, start: 74, type: 'operator', value: '|>' },
    })
    if (!result2) throw new Error('should not happen')
    expect(code.slice(result2?.token?.start, result2?.token?.end)).toEqual('|>')
  })
})

describe('testing pipe operator special', () => {
  test('pipe operator with sketch', () => {
    let code = `sketch mySketch {
  lineTo(2, 3)
  path myPath = lineTo(0, 1)
  lineTo(1,1)
} |> rx(45, %)
`
    const tokens = lexer(code)
    const { body } = abstractSyntaxTree(tokens)
    expect(body).toEqual([
      {
        type: 'VariableDeclaration',
        start: 0,
        end: 90,
        kind: 'sketch',
        declarations: [
          {
            type: 'VariableDeclarator',
            start: 7,
            end: 90,
            id: {
              type: 'Identifier',
              start: 7,
              end: 15,
              name: 'mySketch',
            },
            init: {
              type: 'PipeExpression',
              start: 16,
              end: 90,
              body: [
                {
                  type: 'SketchExpression',
                  start: 16,
                  end: 77,
                  body: {
                    type: 'BlockStatement',
                    start: 16,
                    end: 77,
                    body: [
                      {
                        type: 'ExpressionStatement',
                        start: 20,
                        end: 32,
                        expression: {
                          type: 'CallExpression',
                          start: 20,
                          end: 32,
                          callee: {
                            type: 'Identifier',
                            start: 20,
                            end: 26,
                            name: 'lineTo',
                          },
                          arguments: [
                            {
                              type: 'Literal',
                              start: 27,
                              end: 28,
                              value: 2,
                              raw: '2',
                            },
                            {
                              type: 'Literal',
                              start: 30,
                              end: 31,
                              value: 3,
                              raw: '3',
                            },
                          ],
                          optional: false,
                        },
                      },
                      {
                        type: 'VariableDeclaration',
                        start: 35,
                        end: 61,
                        kind: 'path',
                        declarations: [
                          {
                            type: 'VariableDeclarator',
                            start: 40,
                            end: 61,
                            id: {
                              type: 'Identifier',
                              start: 40,
                              end: 46,
                              name: 'myPath',
                            },
                            init: {
                              type: 'CallExpression',
                              start: 49,
                              end: 61,
                              callee: {
                                type: 'Identifier',
                                start: 49,
                                end: 55,
                                name: 'lineTo',
                              },
                              arguments: [
                                {
                                  type: 'Literal',
                                  start: 56,
                                  end: 57,
                                  value: 0,
                                  raw: '0',
                                },
                                {
                                  type: 'Literal',
                                  start: 59,
                                  end: 60,
                                  value: 1,
                                  raw: '1',
                                },
                              ],
                              optional: false,
                            },
                          },
                        ],
                      },
                      {
                        type: 'ExpressionStatement',
                        start: 64,
                        end: 75,
                        expression: {
                          type: 'CallExpression',
                          start: 64,
                          end: 75,
                          callee: {
                            type: 'Identifier',
                            start: 64,
                            end: 70,
                            name: 'lineTo',
                          },
                          arguments: [
                            {
                              type: 'Literal',
                              start: 71,
                              end: 72,
                              value: 1,
                              raw: '1',
                            },
                            {
                              type: 'Literal',
                              start: 73,
                              end: 74,
                              value: 1,
                              raw: '1',
                            },
                          ],
                          optional: false,
                        },
                      },
                    ],
                  },
                },
                {
                  type: 'CallExpression',
                  start: 81,
                  end: 90,
                  callee: {
                    type: 'Identifier',
                    start: 81,
                    end: 83,
                    name: 'rx',
                  },
                  arguments: [
                    {
                      type: 'Literal',
                      start: 84,
                      end: 86,
                      value: 45,
                      raw: '45',
                    },
                    {
                      type: 'PipeSubstitution',
                      start: 88,
                      end: 89,
                    },
                  ],
                  optional: false,
                },
              ],
            },
          },
        ],
      },
    ])
  })
  test('pipe operator with binary expression', () => {
    let code = `const myVar = 5 + 6 |> myFunc(45, %)`
    const tokens = lexer(code)
    const { body } = abstractSyntaxTree(tokens)
    expect(body).toEqual([
      {
        type: 'VariableDeclaration',
        start: 0,
        end: 36,
        kind: 'const',
        declarations: [
          {
            type: 'VariableDeclarator',
            start: 6,
            end: 36,
            id: {
              type: 'Identifier',
              start: 6,
              end: 11,
              name: 'myVar',
            },
            init: {
              type: 'PipeExpression',
              start: 12,
              end: 36,
              body: [
                {
                  type: 'BinaryExpression',
                  start: 14,
                  end: 19,
                  left: {
                    type: 'Literal',
                    start: 14,
                    end: 15,
                    value: 5,
                    raw: '5',
                  },
                  operator: '+',
                  right: {
                    type: 'Literal',
                    start: 18,
                    end: 19,
                    value: 6,
                    raw: '6',
                  },
                },
                {
                  type: 'CallExpression',
                  start: 23,
                  end: 36,
                  callee: {
                    type: 'Identifier',
                    start: 23,
                    end: 29,
                    name: 'myFunc',
                  },
                  arguments: [
                    {
                      type: 'Literal',
                      start: 30,
                      end: 32,
                      value: 45,
                      raw: '45',
                    },
                    {
                      type: 'PipeSubstitution',
                      start: 34,
                      end: 35,
                    },
                  ],
                  optional: false,
                },
              ],
            },
          },
        ],
      },
    ])
  })
})
