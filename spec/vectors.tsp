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
  },
  #{
    name: "inline_math-lower",
    stage: "callable",
    input: #{ markdown: "Euler: $E = mc^2$ ok.\n" },
    expected: #{
      blocks: #[
        #{
          content: #[
            #{
              data: "Euler: ",
              kind: "text"
            },
            #{
              data: "E = mc^2",
              kind: "text"
            },
            #{
              data: " ok.",
              kind: "text"
            }
          ],
          kind: "paragraph"
        }
      ]
    }
  },
  #{
    name: "display_math-lower",
    stage: "callable",
    input: #{ markdown: "Before\n\n$$\na+b\n$$\n\nAfter\n" },
    expected: #{
      blocks: #[
        #{
          content: #[
            #{
              data: "Before",
              kind: "text"
            }
          ],
          kind: "paragraph"
        },
        #{
          content: #[
            #{
              data: "\na+b\n",
              kind: "text"
            }
          ],
          kind: "paragraph"
        },
        #{
          content: #[
            #{
              data: "After",
              kind: "text"
            }
          ],
          kind: "paragraph"
        }
      ]
    }
  },
  #{
    name: "math_currency-lower",
    stage: "callable",
    input: #{ markdown: "I paid $100 and $200 today.\n" },
    expected: #{
      blocks: #[
        #{
          content: #[
            #{
              data: "I paid $100 and $200 today.",
              kind: "text"
            }
          ],
          kind: "paragraph"
        }
      ]
    }
  },
  #{
    name: "math_digit_after_close-lower",
    stage: "callable",
    input: #{ markdown: "x$a$2\n" },
    expected: #{
      blocks: #[
        #{
          content: #[
            #{
              data: "x$a$2",
              kind: "text"
            }
          ],
          kind: "paragraph"
        }
      ]
    }
  },
  #{
    name: "math_space_boundary-lower",
    stage: "callable",
    input: #{ markdown: "$ x $\n" },
    expected: #{
      blocks: #[
        #{
          content: #[
            #{
              data: "$ x $",
              kind: "text"
            }
          ],
          kind: "paragraph"
        }
      ]
    }
  },
  #{
    name: "math_code-lower",
    stage: "callable",
    input: #{ markdown: "Euler is $`e^{i\\pi}+1=0`$ done.\n" },
    expected: #{
      blocks: #[
        #{
          content: #[
            #{
              data: "Euler is ",
              kind: "text"
            },
            #{
              data: "e^{i\\pi}+1=0",
              kind: "text"
            },
            #{
              data: " done.",
              kind: "text"
            }
          ],
          kind: "paragraph"
        }
      ]
    }
  },
  #{
    name: "superscript-lower",
    stage: "callable",
    input: #{ markdown: "area r^2^ end\n" },
    expected: #{
      blocks: #[
        #{
          content: #[
            #{
              data: "area r",
              kind: "text"
            },
            #{
              data: "2",
              kind: "text"
            },
            #{
              data: " end",
              kind: "text"
            }
          ],
          kind: "paragraph"
        }
      ]
    }
  },
  #{
    name: "superscript_in_strong-lower",
    stage: "callable",
    input: #{ markdown: "A **bold x^2^** term\n" },
    expected: #{
      blocks: #[
        #{
          content: #[
            #{
              data: "A ",
              kind: "text"
            },
            #{
              data: #[
                #{
                  data: "bold x",
                  kind: "text"
                },
                #{
                  data: "2",
                  kind: "text"
                }
              ],
              kind: "strong"
            },
            #{
              data: " term",
              kind: "text"
            }
          ],
          kind: "paragraph"
        }
      ]
    }
  },
  #{
    name: "superscript_emphasis-lower",
    stage: "callable",
    input: #{ markdown: "x^*y*^\n" },
    expected: #{
      blocks: #[
        #{
          content: #[
            #{
              data: "x",
              kind: "text"
            },
            #{
              data: #[
                #{
                  data: "y",
                  kind: "text"
                }
              ],
              kind: "emphasis"
            }
          ],
          kind: "paragraph"
        }
      ]
    }
  },
  #{
    name: "superscript_image_alt-lower",
    stage: "callable",
    input: #{ markdown: "![a^b^](u.png)\n" },
    expected: #{
      blocks: #[
        #{
          content: #[
            #{
              data: "a^b^",
              kind: "text"
            }
          ],
          kind: "paragraph"
        }
      ]
    }
  },
  #{
    name: "superscript_link_label-lower",
    stage: "callable",
    input: #{ markdown: "[note^2^](http://example.com)\n" },
    expected: #{
      blocks: #[
        #{
          content: #[
            #{
              content: #[
                #{
                  data: "note^2^",
                  kind: "text"
                }
              ],
              href: "http://example.com",
              kind: "link"
            }
          ],
          kind: "paragraph"
        }
      ]
    }
  },
  #{
    name: "footnote_ref-lower",
    stage: "callable",
    input: #{ markdown: "text[^1] end\n\n[^1]: note\n" },
    expected: #{
      blocks: #[
        #{
          content: #[
            #{
              data: "text",
              kind: "text"
            },
            #{
              data: " end",
              kind: "text"
            }
          ],
          kind: "paragraph"
        }
      ]
    }
  },
  #{
    name: "footnote_inline-lower",
    stage: "callable",
    input: #{ markdown: "text^[inline note] end\n" },
    expected: #{
      blocks: #[
        #{
          content: #[
            #{
              data: "text^[inline note] end",
              kind: "text"
            }
          ],
          kind: "paragraph"
        }
      ]
    }
  },
  #{
    name: "footnote_case_fold-lower",
    stage: "callable",
    input: #{ markdown: "text[^A] end\n\n[^a]: note\n" },
    expected: #{
      blocks: #[
        #{
          content: #[
            #{
              data: "text",
              kind: "text"
            },
            #{
              data: " end",
              kind: "text"
            }
          ],
          kind: "paragraph"
        }
      ]
    }
  },
  #{
    name: "autolink_url_parens-lower",
    stage: "callable",
    input: #{ markdown: "Visit https://example.com/a_(b).\n" },
    expected: #{
      blocks: #[
        #{
          content: #[
            #{
              data: "Visit ",
              kind: "text"
            },
            #{
              content: #[
                #{
                  data: "https://example.com/a_(b)",
                  kind: "text"
                }
              ],
              href: "https://example.com/a_(b)",
              kind: "link"
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
    name: "autolink_www-lower",
    stage: "callable",
    input: #{ markdown: "See www.example.com/path\n" },
    expected: #{
      blocks: #[
        #{
          content: #[
            #{
              data: "See ",
              kind: "text"
            },
            #{
              content: #[
                #{
                  data: "www.example.com/path",
                  kind: "text"
                }
              ],
              href: "http://www.example.com/path",
              kind: "link"
            }
          ],
          kind: "paragraph"
        }
      ]
    }
  },
  #{
    name: "autolink_email-lower",
    stage: "callable",
    input: #{ markdown: "Mail a.b+tag@example.co.uk.\n" },
    expected: #{
      blocks: #[
        #{
          content: #[
            #{
              data: "Mail ",
              kind: "text"
            },
            #{
              content: #[
                #{
                  data: "a.b+tag@example.co.uk",
                  kind: "text"
                }
              ],
              href: "mailto:a.b+tag@example.co.uk",
              kind: "link"
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
    name: "autolink_domain_reject-lower",
    stage: "callable",
    input: #{ markdown: "http://a.b_c.com\n" },
    expected: #{
      blocks: #[
        #{
          content: #[
            #{
              data: "http://a.b_c.com",
              kind: "text"
            }
          ],
          kind: "paragraph"
        }
      ]
    }
  },
  #{
    name: "autolink_bracket_suppressed-lower",
    stage: "callable",
    input: #{ markdown: "[**x** https://example.com]\n" },
    expected: #{
      blocks: #[
        #{
          content: #[
            #{
              data: "[",
              kind: "text"
            },
            #{
              data: #[
                #{
                  data: "x",
                  kind: "text"
                }
              ],
              kind: "strong"
            },
            #{
              data: " https://example.com]",
              kind: "text"
            }
          ],
          kind: "paragraph"
        }
      ]
    }
  },
  #{
    name: "autolink_www_preceder-lower",
    stage: "callable",
    input: #{ markdown: "`x`www.example.com\n" },
    expected: #{
      blocks: #[
        #{
          content: #[
            #{
              data: "x",
              kind: "code"
            },
            #{
              data: "www.example.com",
              kind: "text"
            }
          ],
          kind: "paragraph"
        }
      ]
    }
  },
  #{
    name: "autolink_footnote_preceder-lower",
    stage: "callable",
    input: #{ markdown: "*x*[^1]www.y.com\n\n[^1]: note\n" },
    expected: #{
      blocks: #[
        #{
          content: #[
            #{
              data: #[
                #{
                  data: "x",
                  kind: "text"
                }
              ],
              kind: "emphasis"
            },
            #{
              data: "www.y.com",
              kind: "text"
            }
          ],
          kind: "paragraph"
        }
      ]
    }
  },
  #{
    name: "alert_note_unwrap-lower",
    stage: "callable",
    input: #{ markdown: "> [!NOTE]\n> This is a note.\n" },
    expected: #{
      blocks: #[
        #{
          content: #[
            #{
              data: "This is a note.",
              kind: "text"
            }
          ],
          kind: "paragraph"
        }
      ]
    }
  },
  #{
    name: "alert_title_dropped-lower",
    stage: "callable",
    input: #{ markdown: "> [!TIP] a title here\n> body text\n" },
    expected: #{
      blocks: #[
        #{
          content: #[
            #{
              data: "body text",
              kind: "text"
            }
          ],
          kind: "paragraph"
        }
      ]
    }
  },
  #{
    name: "alert_unknown_kept-lower",
    stage: "callable",
    input: #{ markdown: "> [!UNKNOWN]\n> body\n" },
    expected: #{
      blocks: #[
        #{
          blocks: #[
            #{
              content: #[
                #{
                  data: "[!UNKNOWN]",
                  kind: "text"
                },
                #{
                  kind: "soft_break"
                },
                #{
                  data: "body",
                  kind: "text"
                }
              ],
              kind: "paragraph"
            }
          ],
          kind: "block_quote"
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
