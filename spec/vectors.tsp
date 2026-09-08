namespace Quilldown;

// Auto-derived from the Rust oracle (lower) + deterministic stats (emit).

const LowerVectors = #[
  #{
    name: "headings-lower",
    stage: "callable",
    input: #{ markdown: "# One\n\n## Two\n\n### Three\n" },
    expected: #{
      blocks: #[
        #{
          content: #[
            #{
              data: "One",
              kind: "text"
            }
          ],
          kind: "heading",
          level: 1
        },
        #{
          content: #[
            #{
              data: "Two",
              kind: "text"
            }
          ],
          kind: "heading",
          level: 2
        },
        #{
          content: #[
            #{
              data: "Three",
              kind: "text"
            }
          ],
          kind: "heading",
          level: 3
        }
      ]
    }
  },
  #{
    name: "paragraph_inline-lower",
    stage: "callable",
    input: #{ markdown: "This is **bold**, *italic*, ~~struck~~, and `code`.\n" },
    expected: #{
      blocks: #[
        #{
          content: #[
            #{
              data: "This is ",
              kind: "text"
            },
            #{
              data: #[
                #{
                  data: "bold",
                  kind: "text"
                }
              ],
              kind: "strong"
            },
            #{
              data: ", ",
              kind: "text"
            },
            #{
              data: #[
                #{
                  data: "italic",
                  kind: "text"
                }
              ],
              kind: "emphasis"
            },
            #{
              data: ", ",
              kind: "text"
            },
            #{
              data: #[
                #{
                  data: "struck",
                  kind: "text"
                }
              ],
              kind: "strikethrough"
            },
            #{
              data: ", and ",
              kind: "text"
            },
            #{
              data: "code",
              kind: "code"
            },
            #{
              data: ".",
              kind: "text"
            }
          ],
          kind: "paragraph"
        }
      ]
    }
  },
  #{
    name: "external_link-lower",
    stage: "callable",
    input: #{ markdown: "See the [manual](https://example.com/guide).\n" },
    expected: #{
      blocks: #[
        #{
          content: #[
            #{
              data: "See the ",
              kind: "text"
            },
            #{
              kind: "link",
              href: "https://example.com/guide",
              content: #[
                #{
                  data: "manual",
                  kind: "text"
                }
              ]
            },
            #{
              data: ".",
              kind: "text"
            }
          ],
          kind: "paragraph"
        }
      ]
    }
  },
  #{
    name: "anchor_link-lower",
    stage: "callable",
    input: #{ markdown: "# Getting Started\n\nJump to [start](#getting-started).\n" },
    expected: #{
      blocks: #[
        #{
          content: #[
            #{
              data: "Getting Started",
              kind: "text"
            }
          ],
          kind: "heading",
          level: 1
        },
        #{
          content: #[
            #{
              data: "Jump to ",
              kind: "text"
            },
            #{
              kind: "link",
              href: "#getting-started",
              content: #[
                #{
                  data: "start",
                  kind: "text"
                }
              ]
            },
            #{
              data: ".",
              kind: "text"
            }
          ],
          kind: "paragraph"
        }
      ]
    }
  },
  #{
    name: "ordered_list-lower",
    stage: "callable",
    input: #{ markdown: "1. alpha\n2. beta\n3. gamma\n" },
    expected: #{
      blocks: #[
        #{
          items: #[
            #{
              kind: "list_item",
              blocks: #[
                #{
                  content: #[
                    #{
                      data: "alpha",
                      kind: "text"
                    }
                  ],
                  kind: "paragraph"
                }
              ]
            },
            #{
              kind: "list_item",
              blocks: #[
                #{
                  content: #[
                    #{
                      data: "beta",
                      kind: "text"
                    }
                  ],
                  kind: "paragraph"
                }
              ]
            },
            #{
              kind: "list_item",
              blocks: #[
                #{
                  content: #[
                    #{
                      data: "gamma",
                      kind: "text"
                    }
                  ],
                  kind: "paragraph"
                }
              ]
            }
          ],
          kind: "list",
          ordered: true,
          start: 1
        }
      ]
    }
  },
  #{
    name: "ordered_start-lower",
    stage: "callable",
    input: #{ markdown: "3. first\n4. second\n" },
    expected: #{
      blocks: #[
        #{
          items: #[
            #{
              kind: "list_item",
              blocks: #[
                #{
                  content: #[
                    #{
                      data: "first",
                      kind: "text"
                    }
                  ],
                  kind: "paragraph"
                }
              ]
            },
            #{
              kind: "list_item",
              blocks: #[
                #{
                  content: #[
                    #{
                      data: "second",
                      kind: "text"
                    }
                  ],
                  kind: "paragraph"
                }
              ]
            }
          ],
          kind: "list",
          ordered: true,
          start: 3
        }
      ]
    }
  },
  #{
    name: "bullet_list-lower",
    stage: "callable",
    input: #{ markdown: "- alpha\n- beta\n" },
    expected: #{
      blocks: #[
        #{
          items: #[
            #{
              kind: "list_item",
              blocks: #[
                #{
                  content: #[
                    #{
                      data: "alpha",
                      kind: "text"
                    }
                  ],
                  kind: "paragraph"
                }
              ]
            },
            #{
              kind: "list_item",
              blocks: #[
                #{
                  content: #[
                    #{
                      data: "beta",
                      kind: "text"
                    }
                  ],
                  kind: "paragraph"
                }
              ]
            }
          ],
          kind: "list",
          ordered: false,
          start: 1
        }
      ]
    }
  },
  #{
    name: "task_list-lower",
    stage: "callable",
    input: #{ markdown: "- [x] done\n- [ ] todo\n" },
    expected: #{
      blocks: #[
        #{
          items: #[
            #{
              kind: "list_item",
              blocks: #[
                #{
                  content: #[
                    #{
                      data: "done",
                      kind: "text"
                    }
                  ],
                  kind: "paragraph"
                }
              ],
              task: true
            },
            #{
              kind: "list_item",
              blocks: #[
                #{
                  content: #[
                    #{
                      data: "todo",
                      kind: "text"
                    }
                  ],
                  kind: "paragraph"
                }
              ],
              task: false
            }
          ],
          kind: "list",
          ordered: false,
          start: 1
        }
      ]
    }
  },
  #{
    name: "table-lower",
    stage: "callable",
    input: #{ markdown: "| Name | Qty |\n|:-----|----:|\n| Pears | 3 |\n| Figs | 12 |\n" },
    expected: #{
      blocks: #[
        #{
          align: #[
            "left",
            "right"
          ],
          head: #{
            cells: #[
              #{
                content: #[
                  #{
                    data: "Name",
                    kind: "text"
                  }
                ]
              },
              #{
                content: #[
                  #{
                    data: "Qty",
                    kind: "text"
                  }
                ]
              }
            ]
          },
          kind: "table",
          rows: #[
            #{
              cells: #[
                #{
                  content: #[
                    #{
                      data: "Pears",
                      kind: "text"
                    }
                  ]
                },
                #{
                  content: #[
                    #{
                      data: "3",
                      kind: "text"
                    }
                  ]
                }
              ]
            },
            #{
              cells: #[
                #{
                  content: #[
                    #{
                      data: "Figs",
                      kind: "text"
                    }
                  ]
                },
                #{
                  content: #[
                    #{
                      data: "12",
                      kind: "text"
                    }
                  ]
                }
              ]
            }
          ]
        }
      ]
    }
  },
  #{
    name: "code_block-lower",
    stage: "callable",
    input: #{ markdown: "```rust\nfn main() {}\n```\n" },
    expected: #{
      blocks: #[
        #{
          code: "fn main() {}\n",
          kind: "code_block",
          language: "rust"
        }
      ]
    }
  },
  #{
    name: "blockquote-lower",
    stage: "callable",
    input: #{ markdown: "> quoted **text**\n" },
    expected: #{
      blocks: #[
        #{
          blocks: #[
            #{
              content: #[
                #{
                  data: "quoted ",
                  kind: "text"
                },
                #{
                  data: #[
                    #{
                      data: "text",
                      kind: "text"
                    }
                  ],
                  kind: "strong"
                }
              ],
              kind: "paragraph"
            }
          ],
          kind: "block_quote"
        }
      ]
    }
  },
  #{
    name: "thematic_break-lower",
    stage: "callable",
    input: #{ markdown: "before\n\n---\n\nafter\n" },
    expected: #{
      blocks: #[
        #{
          content: #[
            #{
              data: "before",
              kind: "text"
            }
          ],
          kind: "paragraph"
        },
        #{
          kind: "thematic_break"
        },
        #{
          content: #[
            #{
              data: "after",
              kind: "text"
            }
          ],
          kind: "paragraph"
        }
      ]
    }
  },
  #{
    name: "heading_para-lower",
    stage: "callable",
    input: #{ markdown: "# Title\n\nHello **world**.\n" },
    expected: #{
      blocks: #[
        #{
          content: #[
            #{
              data: "Title",
              kind: "text"
            }
          ],
          kind: "heading",
          level: 1
        },
        #{
          content: #[
            #{
              data: "Hello ",
              kind: "text"
            },
            #{
              data: #[
                #{
                  data: "world",
                  kind: "text"
                }
              ],
              kind: "strong"
            },
            #{
              data: ".",
              kind: "text"
            }
          ],
          kind: "paragraph"
        }
      ]
    }
  },
  #{
    name: "image_alt-lower",
    stage: "callable",
    input: #{ markdown: "See ![a cat](cat.png) here.\n" },
    expected: #{
      blocks: #[
        #{
          content: #[
            #{
              data: "See ",
              kind: "text"
            },
            #{
              data: "a cat",
              kind: "text"
            },
            #{
              data: " here.",
              kind: "text"
            }
          ],
          kind: "paragraph"
        }
      ]
    }
  }
];

const EmitVectors = #[
  #{
    name: "headings-emit",
    stage: "callable",
    input: #{ doc: #{
      blocks: #[
        #{
          content: #[
            #{
              data: "One",
              kind: "text"
            }
          ],
          kind: "heading",
          level: 1
        },
        #{
          content: #[
            #{
              data: "Two",
              kind: "text"
            }
          ],
          kind: "heading",
          level: 2
        },
        #{
          content: #[
            #{
              data: "Three",
              kind: "text"
            }
          ],
          kind: "heading",
          level: 3
        }
      ]
    } },
    expected: #{
      headings: 3,
      paragraphs: 0,
      codeBlocks: 0,
      blockQuotes: 0,
      lists: 0,
      listItems: 0,
      tables: 0,
      links: 0,
      thematicBreaks: 0,
      warnings: #[]
    }
  },
  #{
    name: "paragraph_inline-emit",
    stage: "callable",
    input: #{ doc: #{
      blocks: #[
        #{
          content: #[
            #{
              data: "This is ",
              kind: "text"
            },
            #{
              data: #[
                #{
                  data: "bold",
                  kind: "text"
                }
              ],
              kind: "strong"
            },
            #{
              data: ", ",
              kind: "text"
            },
            #{
              data: #[
                #{
                  data: "italic",
                  kind: "text"
                }
              ],
              kind: "emphasis"
            },
            #{
              data: ", ",
              kind: "text"
            },
            #{
              data: #[
                #{
                  data: "struck",
                  kind: "text"
                }
              ],
              kind: "strikethrough"
            },
            #{
              data: ", and ",
              kind: "text"
            },
            #{
              data: "code",
              kind: "code"
            },
            #{
              data: ".",
              kind: "text"
            }
          ],
          kind: "paragraph"
        }
      ]
    } },
    expected: #{
      headings: 0,
      paragraphs: 1,
      codeBlocks: 0,
      blockQuotes: 0,
      lists: 0,
      listItems: 0,
      tables: 0,
      links: 0,
      thematicBreaks: 0,
      warnings: #[]
    }
  },
  #{
    name: "external_link-emit",
    stage: "callable",
    input: #{ doc: #{
      blocks: #[
        #{
          content: #[
            #{
              data: "See the ",
              kind: "text"
            },
            #{
              kind: "link",
              href: "https://example.com/guide",
              content: #[
                #{
                  data: "manual",
                  kind: "text"
                }
              ]
            },
            #{
              data: ".",
              kind: "text"
            }
          ],
          kind: "paragraph"
        }
      ]
    } },
    expected: #{
      headings: 0,
      paragraphs: 1,
      codeBlocks: 0,
      blockQuotes: 0,
      lists: 0,
      listItems: 0,
      tables: 0,
      links: 1,
      thematicBreaks: 0,
      warnings: #[]
    }
  },
  #{
    name: "anchor_link-emit",
    stage: "callable",
    input: #{ doc: #{
      blocks: #[
        #{
          content: #[
            #{
              data: "Getting Started",
              kind: "text"
            }
          ],
          kind: "heading",
          level: 1
        },
        #{
          content: #[
            #{
              data: "Jump to ",
              kind: "text"
            },
            #{
              kind: "link",
              href: "#getting-started",
              content: #[
                #{
                  data: "start",
                  kind: "text"
                }
              ]
            },
            #{
              data: ".",
              kind: "text"
            }
          ],
          kind: "paragraph"
        }
      ]
    } },
    expected: #{
      headings: 1,
      paragraphs: 1,
      codeBlocks: 0,
      blockQuotes: 0,
      lists: 0,
      listItems: 0,
      tables: 0,
      links: 1,
      thematicBreaks: 0,
      warnings: #[]
    }
  },
  #{
    name: "ordered_list-emit",
    stage: "callable",
    input: #{ doc: #{
      blocks: #[
        #{
          items: #[
            #{
              kind: "list_item",
              blocks: #[
                #{
                  content: #[
                    #{
                      data: "alpha",
                      kind: "text"
                    }
                  ],
                  kind: "paragraph"
                }
              ]
            },
            #{
              kind: "list_item",
              blocks: #[
                #{
                  content: #[
                    #{
                      data: "beta",
                      kind: "text"
                    }
                  ],
                  kind: "paragraph"
                }
              ]
            },
            #{
              kind: "list_item",
              blocks: #[
                #{
                  content: #[
                    #{
                      data: "gamma",
                      kind: "text"
                    }
                  ],
                  kind: "paragraph"
                }
              ]
            }
          ],
          kind: "list",
          ordered: true,
          start: 1
        }
      ]
    } },
    expected: #{
      headings: 0,
      paragraphs: 3,
      codeBlocks: 0,
      blockQuotes: 0,
      lists: 1,
      listItems: 3,
      tables: 0,
      links: 0,
      thematicBreaks: 0,
      warnings: #[]
    }
  },
  #{
    name: "ordered_start-emit",
    stage: "callable",
    input: #{ doc: #{
      blocks: #[
        #{
          items: #[
            #{
              kind: "list_item",
              blocks: #[
                #{
                  content: #[
                    #{
                      data: "first",
                      kind: "text"
                    }
                  ],
                  kind: "paragraph"
                }
              ]
            },
            #{
              kind: "list_item",
              blocks: #[
                #{
                  content: #[
                    #{
                      data: "second",
                      kind: "text"
                    }
                  ],
                  kind: "paragraph"
                }
              ]
            }
          ],
          kind: "list",
          ordered: true,
          start: 3
        }
      ]
    } },
    expected: #{
      headings: 0,
      paragraphs: 2,
      codeBlocks: 0,
      blockQuotes: 0,
      lists: 1,
      listItems: 2,
      tables: 0,
      links: 0,
      thematicBreaks: 0,
      warnings: #[]
    }
  },
  #{
    name: "bullet_list-emit",
    stage: "callable",
    input: #{ doc: #{
      blocks: #[
        #{
          items: #[
            #{
              kind: "list_item",
              blocks: #[
                #{
                  content: #[
                    #{
                      data: "alpha",
                      kind: "text"
                    }
                  ],
                  kind: "paragraph"
                }
              ]
            },
            #{
              kind: "list_item",
              blocks: #[
                #{
                  content: #[
                    #{
                      data: "beta",
                      kind: "text"
                    }
                  ],
                  kind: "paragraph"
                }
              ]
            }
          ],
          kind: "list",
          ordered: false,
          start: 1
        }
      ]
    } },
    expected: #{
      headings: 0,
      paragraphs: 2,
      codeBlocks: 0,
      blockQuotes: 0,
      lists: 1,
      listItems: 2,
      tables: 0,
      links: 0,
      thematicBreaks: 0,
      warnings: #[]
    }
  },
  #{
    name: "task_list-emit",
    stage: "callable",
    input: #{ doc: #{
      blocks: #[
        #{
          items: #[
            #{
              kind: "list_item",
              blocks: #[
                #{
                  content: #[
                    #{
                      data: "done",
                      kind: "text"
                    }
                  ],
                  kind: "paragraph"
                }
              ],
              task: true
            },
            #{
              kind: "list_item",
              blocks: #[
                #{
                  content: #[
                    #{
                      data: "todo",
                      kind: "text"
                    }
                  ],
                  kind: "paragraph"
                }
              ],
              task: false
            }
          ],
          kind: "list",
          ordered: false,
          start: 1
        }
      ]
    } },
    expected: #{
      headings: 0,
      paragraphs: 2,
      codeBlocks: 0,
      blockQuotes: 0,
      lists: 1,
      listItems: 2,
      tables: 0,
      links: 0,
      thematicBreaks: 0,
      warnings: #[]
    }
  },
  #{
    name: "table-emit",
    stage: "callable",
    input: #{ doc: #{
      blocks: #[
        #{
          align: #[
            "left",
            "right"
          ],
          head: #{
            cells: #[
              #{
                content: #[
                  #{
                    data: "Name",
                    kind: "text"
                  }
                ]
              },
              #{
                content: #[
                  #{
                    data: "Qty",
                    kind: "text"
                  }
                ]
              }
            ]
          },
          kind: "table",
          rows: #[
            #{
              cells: #[
                #{
                  content: #[
                    #{
                      data: "Pears",
                      kind: "text"
                    }
                  ]
                },
                #{
                  content: #[
                    #{
                      data: "3",
                      kind: "text"
                    }
                  ]
                }
              ]
            },
            #{
              cells: #[
                #{
                  content: #[
                    #{
                      data: "Figs",
                      kind: "text"
                    }
                  ]
                },
                #{
                  content: #[
                    #{
                      data: "12",
                      kind: "text"
                    }
                  ]
                }
              ]
            }
          ]
        }
      ]
    } },
    expected: #{
      headings: 0,
      paragraphs: 0,
      codeBlocks: 0,
      blockQuotes: 0,
      lists: 0,
      listItems: 0,
      tables: 1,
      links: 0,
      thematicBreaks: 0,
      warnings: #[]
    }
  },
  #{
    name: "code_block-emit",
    stage: "callable",
    input: #{ doc: #{
      blocks: #[
        #{
          code: "fn main() {}\n",
          kind: "code_block",
          language: "rust"
        }
      ]
    } },
    expected: #{
      headings: 0,
      paragraphs: 0,
      codeBlocks: 1,
      blockQuotes: 0,
      lists: 0,
      listItems: 0,
      tables: 0,
      links: 0,
      thematicBreaks: 0,
      warnings: #[]
    }
  },
  #{
    name: "blockquote-emit",
    stage: "callable",
    input: #{ doc: #{
      blocks: #[
        #{
          blocks: #[
            #{
              content: #[
                #{
                  data: "quoted ",
                  kind: "text"
                },
                #{
                  data: #[
                    #{
                      data: "text",
                      kind: "text"
                    }
                  ],
                  kind: "strong"
                }
              ],
              kind: "paragraph"
            }
          ],
          kind: "block_quote"
        }
      ]
    } },
    expected: #{
      headings: 0,
      paragraphs: 1,
      codeBlocks: 0,
      blockQuotes: 1,
      lists: 0,
      listItems: 0,
      tables: 0,
      links: 0,
      thematicBreaks: 0,
      warnings: #[]
    }
  },
  #{
    name: "thematic_break-emit",
    stage: "callable",
    input: #{ doc: #{
      blocks: #[
        #{
          content: #[
            #{
              data: "before",
              kind: "text"
            }
          ],
          kind: "paragraph"
        },
        #{
          kind: "thematic_break"
        },
        #{
          content: #[
            #{
              data: "after",
              kind: "text"
            }
          ],
          kind: "paragraph"
        }
      ]
    } },
    expected: #{
      headings: 0,
      paragraphs: 2,
      codeBlocks: 0,
      blockQuotes: 0,
      lists: 0,
      listItems: 0,
      tables: 0,
      links: 0,
      thematicBreaks: 1,
      warnings: #[]
    }
  },
  #{
    name: "heading_para-emit",
    stage: "callable",
    input: #{ doc: #{
      blocks: #[
        #{
          content: #[
            #{
              data: "Title",
              kind: "text"
            }
          ],
          kind: "heading",
          level: 1
        },
        #{
          content: #[
            #{
              data: "Hello ",
              kind: "text"
            },
            #{
              data: #[
                #{
                  data: "world",
                  kind: "text"
                }
              ],
              kind: "strong"
            },
            #{
              data: ".",
              kind: "text"
            }
          ],
          kind: "paragraph"
        }
      ]
    } },
    expected: #{
      headings: 1,
      paragraphs: 1,
      codeBlocks: 0,
      blockQuotes: 0,
      lists: 0,
      listItems: 0,
      tables: 0,
      links: 0,
      thematicBreaks: 0,
      warnings: #[]
    }
  }
];
