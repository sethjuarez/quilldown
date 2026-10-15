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
    name: "code_block_crlf-lower",
    stage: "callable",
    input: #{ markdown: "```rust\r\nfn main() {}\r\n```\r\n" },
    expected: #{
      blocks: #[
        #{
          code: "fn main() {}\r\n",
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
    name: "alert-lower",
    stage: "callable",
    input: #{ markdown: "> [!WARNING] Contract risk\n> Review **pricing** before signing.\n" },
    expected: #{
      blocks: #[
        #{
          alert_type: "warning",
          blocks: #[
            #{
              content: #[
                #{
                  data: "Review ",
                  kind: "text"
                },
                #{
                  data: #[
                    #{
                      data: "pricing",
                      kind: "text"
                    }
                  ],
                  kind: "strong"
                },
                #{
                  data: " before signing.",
                  kind: "text"
                }
              ],
              kind: "paragraph"
            }
          ],
          kind: "alert",
          title: "Contract risk"
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
    name: "front_matter-lower",
    stage: "callable",
    input: #{ markdown: "---\ntitle: x\n---\n\nbody\n" },
    expected: #{
      metadata: #{ title: "x" },
      blocks: #[
        #{
          content: #[
            #{
              data: "body",
              kind: "text"
            }
          ],
          kind: "paragraph"
        }
      ]
    }
  },
  #{
    name: "front_matter_before_heading-lower",
    stage: "callable",
    input: #{ markdown: "---\nk: v\n---\n\n# H\n\npara\n" },
    expected: #{
      blocks: #[
        #{
          content: #[
            #{
              data: "H",
              kind: "text"
            }
          ],
          kind: "heading",
          level: 1
        },
        #{
          content: #[
            #{
              data: "para",
              kind: "text"
            }
          ],
          kind: "paragraph"
        }
      ]
    }
  },
  #{
    name: "front_matter_no_close_literal-lower",
    stage: "callable",
    input: #{ markdown: "---\ntitle: x\nbody\n" },
    expected: #{
      blocks: #[
        #{
          kind: "thematic_break"
        },
        #{
          content: #[
            #{
              data: "title: x",
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
              alt: "a cat",
              kind: "image",
              src: "cat.png",
              title: ""
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
              display: false,
              kind: "math",
              latex: "E = mc^2"
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
              display: true,
              kind: "math",
              latex: "\na+b\n"
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
    name: "display_math_crlf-lower",
    stage: "callable",
    input: #{ markdown: "$$\r\na+b\r\n$$\r\n" },
    expected: #{
      blocks: #[
        #{
          content: #[
            #{
              display: true,
              kind: "math",
              latex: "\r\na+b\r\n"
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
              display: false,
              kind: "math",
              latex: "e^{i\\pi}+1=0"
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
    name: "subscript-lower",
    stage: "callable",
    input: #{ markdown: "H~2~O\n" },
    expected: #{
      blocks: #[
        #{
          content: #[
            #{
              data: "H",
              kind: "text"
            },
            #{
              data: #[
                #{
                  data: "2",
                  kind: "text"
                }
              ],
              kind: "subscript"
            },
            #{
              data: "O",
              kind: "text"
            }
          ],
          kind: "paragraph"
        }
      ]
    }
  },
  #{
    name: "subscript_in_strikethrough-lower",
    stage: "callable",
    input: #{ markdown: "~~H~2~O~~\n" },
    expected: #{
      blocks: #[
        #{
          content: #[
            #{
              data: #[
                #{
                  data: "H",
                  kind: "text"
                },
                #{
                  data: #[
                    #{
                      data: "2",
                      kind: "text"
                    }
                  ],
                  kind: "subscript"
                },
                #{
                  data: "O",
                  kind: "text"
                }
              ],
              kind: "strikethrough"
            }
          ],
          kind: "paragraph"
        }
      ]
    }
  },
  #{
    name: "subscript_mismatch_literal-lower",
    stage: "callable",
    input: #{ markdown: "~~a~b!~d~~\n" },
    expected: #{
      blocks: #[
        #{
          content: #[
            #{
              data: "~~a~b!~d~~",
              kind: "text"
            }
          ],
          kind: "paragraph"
        }
      ]
    }
  },
  #{
    name: "subscript_punct_bypass-lower",
    stage: "callable",
    input: #{ markdown: "x~-1~\n" },
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
                  data: "-1",
                  kind: "text"
                }
              ],
              kind: "subscript"
            }
          ],
          kind: "paragraph"
        }
      ]
    }
  },
  #{
    name: "subscript_interior_star-lower",
    stage: "callable",
    input: #{ markdown: "~a*b~c*\n" },
    expected: #{
      blocks: #[
        #{
          content: #[
            #{
              data: #[
                #{
                  data: "a*b",
                  kind: "text"
                }
              ],
              kind: "subscript"
            },
            #{
              data: "c*",
              kind: "text"
            }
          ],
          kind: "paragraph"
        }
      ]
    }
  },
  #{
    name: "subscript_termination_emphasis-lower",
    stage: "callable",
    input: #{ markdown: "~a~~ *b*\n" },
    expected: #{
      blocks: #[
        #{
          content: #[
            #{
              data: "~a~~ *b*",
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
              alt: "a^b^",
              kind: "image",
              src: "u.png",
              title: ""
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
              label: "1",
              kind: "footnote_reference"
            },
            #{
              data: " end",
              kind: "text"
            }
          ],
          kind: "paragraph"
        }
      ],
      footnotes: #[
        #{
          label: "1",
          blocks: #[
            #{
              content: #[
                #{
                  data: "note",
                  kind: "text"
                }
              ],
              kind: "paragraph"
            }
          ]
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
              label: "a",
              kind: "footnote_reference"
            },
            #{
              data: " end",
              kind: "text"
            }
          ],
          kind: "paragraph"
        }
      ],
      footnotes: #[
        #{
          label: "a",
          blocks: #[
            #{
              content: #[
                #{
                  data: "note",
                  kind: "text"
                }
              ],
              kind: "paragraph"
            }
          ]
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
              label: "1",
              kind: "footnote_reference"
            },
            #{
              data: "www.y.com",
              kind: "text"
            }
          ],
          kind: "paragraph"
        }
      ],
      footnotes: #[
        #{
          label: "1",
          blocks: #[
            #{
              content: #[
                #{
                  data: "note",
                  kind: "text"
                }
              ],
              kind: "paragraph"
            }
          ]
        }
      ]
    }
  },
  #{
    name: "alert_note-lower",
    stage: "callable",
    input: #{ markdown: "> [!NOTE]\n> This is a note.\n" },
    expected: #{
      blocks: #[
        #{
          alert_type: "note",
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
          ],
          kind: "alert"
        }
      ]
    }
  },
  #{
    name: "alert_title-lower",
    stage: "callable",
    input: #{ markdown: "> [!TIP] a &amp; \\[title\\]\n> body text\n" },
    expected: #{
      blocks: #[
        #{
          alert_type: "tip",
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
          ],
          kind: "alert",
          title: "a & [title]"
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
  },
  #{
    name: "cross_marker_flanking_skip-lower",
    stage: "callable",
    input: #{
      markdown: "H*~2~*O\n"
    },
    expected: #{
      blocks: #[
        #{
          content: #[
            #{
              data: "H",
              kind: "text"
            },
            #{
              data: #[
                #{
                  data: #[
                    #{
                      data: "2",
                      kind: "text"
                    }
                  ],
                  kind: "subscript"
                }
              ],
              kind: "emphasis"
            },
            #{
              data: "O",
              kind: "text"
            }
          ],
          kind: "paragraph"
        }
      ]
    }
  },
  #{
    name: "cross_marker_crossing_span-lower",
    stage: "callable",
    input: #{
      markdown: "This is *important ~but unfinished* subscript~ text.\n"
    },
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
                  data: "important ~but unfinished",
                  kind: "text"
                }
              ],
              kind: "emphasis"
            },
            #{
              data: " subscript~ text.",
              kind: "text"
            }
          ],
          kind: "paragraph"
        }
      ]
    }
  },
  #{
    name: "cross_marker_repair-lower",
    stage: "callable",
    input: #{
      markdown: "*a ~b* c~\n"
    },
    expected: #{
      blocks: #[
        #{
          content: #[
            #{
              data: #[
                #{
                  data: "a ~b",
                  kind: "text"
                }
              ],
              kind: "emphasis"
            },
            #{
              data: " c~",
              kind: "text"
            }
          ],
          kind: "paragraph"
        }
      ]
    }
  },
  #{
    name: "superscript_nested_emphasis-lower",
    stage: "callable",
    input: #{
      markdown: "x^*y*^\n"
    },
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
    name: "cross_marker_interleaved-lower",
    stage: "callable",
    input: #{
      markdown: "a*b~c*d~\n"
    },
    expected: #{
      blocks: #[
        #{
          content: #[
            #{
              data: "a",
              kind: "text"
            },
            #{
              data: #[
                #{
                  data: "b~c",
                  kind: "text"
                }
              ],
              kind: "emphasis"
            },
            #{
              data: "d~",
              kind: "text"
            }
          ],
          kind: "paragraph"
        }
      ]
    }
  },
  #{
    name: "hard_break_spaces-lower",
    stage: "callable",
    input: #{
      markdown: "line one  \nline two\n"
    },
    expected: #{
      blocks: #[
        #{
          content: #[
            #{
              data: "line one",
              kind: "text"
            },
            #{
              kind: "hard_break"
            },
            #{
              data: "line two",
              kind: "text"
            }
          ],
          kind: "paragraph"
        }
      ]
    }
  },
  #{
    name: "hard_break_backslash-lower",
    stage: "callable",
    input: #{
      markdown: "line one\\\nline two\n"
    },
    expected: #{
      blocks: #[
        #{
          content: #[
            #{
              data: "line one",
              kind: "text"
            },
            #{
              kind: "hard_break"
            },
            #{
              data: "line two",
              kind: "text"
            }
          ],
          kind: "paragraph"
        }
      ]
    }
  },
  #{
    name: "subscript-lower",
    stage: "callable",
    input: #{ markdown: "H~2~O\n" },
    expected: #{
      blocks: #[
        #{
          content: #[
            #{
              data: "H",
              kind: "text"
            },
            #{
              data: #[
                #{
                  data: "2",
                  kind: "text"
                }
              ],
              kind: "subscript"
            },
            #{
              data: "O",
              kind: "text"
            }
          ],
          kind: "paragraph"
        }
      ]
    }
  },
  #{
    name: "footnote-lower",
    stage: "callable",
    input: #{ markdown: "See this[^Note].\n\n[^note]: Foot *note*.\n" },
    expected: #{
      blocks: #[
        #{
          content: #[
            #{
              data: "See this",
              kind: "text"
            },
            #{
              label: "note",
              kind: "footnote_reference"
            },
            #{
              data: ".",
              kind: "text"
            }
          ],
          kind: "paragraph"
        }
      ],
      footnotes: #[
        #{
          label: "note",
          blocks: #[
            #{
              content: #[
                #{
                  data: "Foot ",
                  kind: "text"
                },
                #{
                  data: #[
                    #{
                      data: "note",
                      kind: "text"
                    }
                  ],
                  kind: "emphasis"
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
      ]
    }
  },
  #{
    name: "fenced_math_code_block-lower",
    stage: "callable",
    input: #{ markdown: "```math\nx^2 + y^2\n```\n" },
    expected: #{
      blocks: #[
        #{
          content: #[
            #{
              display: true,
              kind: "math",
              latex: "x^2 + y^2\n"
            }
          ],
          kind: "paragraph"
        }
      ]
    }
  },
  #{
    name: "raw_inline_html-lower",
    stage: "callable",
    input: #{ markdown: "before <span class=\"x\">mid</span> after\n" },
    expected: #{
      blocks: #[
        #{
          content: #[
            #{
              data: "before ",
              kind: "text"
            },
            #{
              data: "mid",
              kind: "text"
            },
            #{
              data: " after",
              kind: "text"
            }
          ],
          kind: "paragraph"
        }
      ]
    }
  },
  #{
    name: "raw_html_block-lower",
    stage: "callable",
    input: #{ markdown: "<div class=\"note\">\n  hi\n</div>\n" },
    expected: #{
      blocks: #[]
    }
  },
  #{
    name: "raw_html_br-lower",
    stage: "callable",
    input: #{ markdown: "line one<br>line two\n" },
    expected: #{
      blocks: #[
        #{
          content: #[
            #{
              data: "line one",
              kind: "text"
            },
            #{
              data: "line two",
              kind: "text"
            }
          ],
          kind: "paragraph"
        }
      ]
    }
  }
];

const WordEmitVectors = #[
  #{
    name: "word_heading_link_manifest-emit",
    stage: "callable",
    input: #{
      doc: #{
        metadata: #{
          title: "Demo",
          creator: "Quilldown",
          language: "de-DE"
        },
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
                data: "Read the ",
                kind: "text"
              },
              #{
                kind: "link",
                href: "https://example.com/guide",
                content: #[
                  #{
                    data: "guide",
                    kind: "text"
                  }
                ]
              },
              #{
                data: " and jump to ",
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
    expected: #{
      stats: #{
        headings: 1,
        paragraphs: 1,
        codeBlocks: 0,
        blockQuotes: 0,
        lists: 0,
        listItems: 0,
        tables: 0,
        links: 2,
        thematicBreaks: 0,
        warnings: #[]
      },
      manifest: #{
        kind: "docx",
        parts: #[
          #{
            name: "/docProps/core.xml",
            contentType: "application/vnd.openxmlformats-package.core-properties+xml",
            role: "core-properties"
          },
          #{
            name: "/word/document.xml",
            contentType: "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml",
            role: "document"
          },
          #{
            name: "/word/styles.xml",
            contentType: "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml",
            role: "styles"
          }
        ],
        relationships: #[
          #{
            relationships: #[
              #{
                type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument",
                target: "word/document.xml"
              },
              #{
                type: "http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties",
                target: "docProps/core.xml"
              }
            ]
          },
          #{
            sourcePart: "/word/document.xml",
            relationships: #[
              #{
                type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink",
                target: "https://example.com/guide",
                targetMode: "External"
              }
            ]
          }
        ],
        contentTypes: #[
          #{
            extension: "rels",
            contentType: "application/vnd.openxmlformats-package.relationships+xml"
          },
          #{
            extension: "xml",
            contentType: "application/xml"
          },
          #{
            partName: "/docProps/core.xml",
            contentType: "application/vnd.openxmlformats-package.core-properties+xml"
          },
          #{
            partName: "/word/document.xml",
            contentType: "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"
          },
          #{
            partName: "/word/styles.xml",
            contentType: "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml"
          }
        ],
        coreProperties: #{
          title: "Demo",
          creator: "Quilldown",
          language: "de-DE"
        }
      },
      invariants: #{
        allRelationshipIdsResolve: true,
        allContentTypesDeclared: true,
        noDuplicateRelationshipIds: true,
        allMediaPartsReachable: true,
        noOrphanRequiredParts: true,
        allHyperlinksHaveRelationships: true,
        allAnchorLinksHaveBookmarks: true,
        allNumberingReferencesDefined: true,
        allTableHeadersMarked: true,
        defaultProofingLanguageSet: true
      },
      warnings: #[]
    }
  },
  #{
    name: "word_dangling_anchor_warning-emit",
    stage: "callable",
    input: #{
      doc: #{
        blocks: #[
          #{
            kind: "paragraph",
            content: #[
              #{ kind: "text", data: "Jump to " },
              #{ kind: "link", href: "#missing", content: #[#{ kind: "text", data: "missing" }] },
              #{ kind: "text", data: "." }
            ]
          }
        ]
      }
    },
    expected: #{
      stats: #{
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
      },
      manifest: #{
        kind: "docx",
        parts: #[
          #{ name: "/docProps/core.xml", contentType: "application/vnd.openxmlformats-package.core-properties+xml", role: "core-properties" },
          #{ name: "/word/document.xml", contentType: "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml", role: "document" },
          #{ name: "/word/styles.xml", contentType: "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml", role: "styles" }
        ],
        relationships: #[
          #{
            relationships: #[
              #{ type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument", target: "word/document.xml" },
              #{ type: "http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties", target: "docProps/core.xml" }
            ]
          }
        ],
        contentTypes: #[
          #{ extension: "rels", contentType: "application/vnd.openxmlformats-package.relationships+xml" },
          #{ extension: "xml", contentType: "application/xml" },
          #{ partName: "/docProps/core.xml", contentType: "application/vnd.openxmlformats-package.core-properties+xml" },
          #{ partName: "/word/document.xml", contentType: "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml" },
          #{ partName: "/word/styles.xml", contentType: "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml" }
        ]
      },
      invariants: #{
        allRelationshipIdsResolve: true,
        allContentTypesDeclared: true,
        noDuplicateRelationshipIds: true,
        allMediaPartsReachable: true,
        noOrphanRequiredParts: true,
        allHyperlinksHaveRelationships: true,
        allAnchorLinksHaveBookmarks: false,
        allNumberingReferencesDefined: true,
        allTableHeadersMarked: true
      },
      warnings: #[
        #{
          code: "word.anchors-unresolved",
          severity: "warning",
          message: "Word invariant failed: allAnchorLinksHaveBookmarks.",
          path: "invariants.allAnchorLinksHaveBookmarks"
        }
      ]
    }
  },
  #{
    name: "word_dangling_anchor_strict-emit",
    stage: "callable",
    input: #{
      doc: #{
        blocks: #[
          #{
            kind: "paragraph",
            content: #[
              #{ kind: "text", data: "Jump to " },
              #{ kind: "link", href: "#missing", content: #[#{ kind: "text", data: "missing" }] },
              #{ kind: "text", data: "." }
            ]
          }
        ]
      },
      options: #{ strict: true }
    },
    expectedError: #{ message: "Word invariant failed: allAnchorLinksHaveBookmarks." }
  },
  #{
    name: "word_image_fallback_media_warning-emit",
    stage: "callable",
    input: #{
      doc: #{
        blocks: #[
          #{
            kind: "paragraph",
            content: #[
              #{ kind: "text", data: "See " },
              #{ kind: "image", src: "cat.png", alt: "a cat", title: "" },
              #{ kind: "text", data: "." }
            ]
          }
        ]
      }
    },
    expected: #{
      stats: #{
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
      },
      manifest: #{
        kind: "docx",
        parts: #[
          #{ name: "/docProps/core.xml", contentType: "application/vnd.openxmlformats-package.core-properties+xml", role: "core-properties" },
          #{ name: "/word/document.xml", contentType: "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml", role: "document" },
          #{ name: "/word/styles.xml", contentType: "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml", role: "styles" }
        ],
        relationships: #[
          #{
            relationships: #[
              #{ type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument", target: "word/document.xml" },
              #{ type: "http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties", target: "docProps/core.xml" }
            ]
          }
        ],
        contentTypes: #[
          #{ extension: "rels", contentType: "application/vnd.openxmlformats-package.relationships+xml" },
          #{ extension: "xml", contentType: "application/xml" },
          #{ partName: "/docProps/core.xml", contentType: "application/vnd.openxmlformats-package.core-properties+xml" },
          #{ partName: "/word/document.xml", contentType: "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml" },
          #{ partName: "/word/styles.xml", contentType: "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml" }
        ]
      },
      invariants: #{
        allRelationshipIdsResolve: true,
        allContentTypesDeclared: true,
        noDuplicateRelationshipIds: true,
        allMediaPartsReachable: false,
        noOrphanRequiredParts: true,
        allHyperlinksHaveRelationships: true,
        allAnchorLinksHaveBookmarks: true,
        allNumberingReferencesDefined: true,
        allTableHeadersMarked: true
      },
      warnings: #[
        #{
          code: "word.media-unreachable",
          severity: "warning",
          message: "Word invariant failed: allMediaPartsReachable.",
          path: "invariants.allMediaPartsReachable"
        }
      ]
    }
  },
  #{
    name: "word_ordered_list_numbering_manifest-emit",
    stage: "callable",
    input: #{
      doc: #{
        blocks: #[
          #{
            kind: "list",
            ordered: true,
            start: 3,
            items: #[
              #{ kind: "list_item", blocks: #[#{ kind: "paragraph", content: #[#{ kind: "text", data: "alpha" }] }] },
              #{ kind: "list_item", blocks: #[#{ kind: "paragraph", content: #[#{ kind: "text", data: "beta" }] }] }
            ]
          }
        ]
      }
    },
    expected: #{
      stats: #{
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
      },
      manifest: #{
        kind: "docx",
        parts: #[
          #{ name: "/docProps/core.xml", contentType: "application/vnd.openxmlformats-package.core-properties+xml", role: "core-properties" },
          #{ name: "/word/document.xml", contentType: "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml", role: "document" },
          #{ name: "/word/numbering.xml", contentType: "application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml", role: "numbering" },
          #{ name: "/word/styles.xml", contentType: "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml", role: "styles" }
        ],
        relationships: #[
          #{
            relationships: #[
              #{ type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument", target: "word/document.xml" },
              #{ type: "http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties", target: "docProps/core.xml" }
            ]
          }
        ],
        contentTypes: #[
          #{ extension: "rels", contentType: "application/vnd.openxmlformats-package.relationships+xml" },
          #{ extension: "xml", contentType: "application/xml" },
          #{ partName: "/docProps/core.xml", contentType: "application/vnd.openxmlformats-package.core-properties+xml" },
          #{ partName: "/word/document.xml", contentType: "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml" },
          #{ partName: "/word/numbering.xml", contentType: "application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml" },
          #{ partName: "/word/styles.xml", contentType: "application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml" }
        ]
      },
      invariants: #{
        allRelationshipIdsResolve: true,
        allContentTypesDeclared: true,
        noDuplicateRelationshipIds: true,
        allMediaPartsReachable: true,
        noOrphanRequiredParts: true,
        allHyperlinksHaveRelationships: true,
        allAnchorLinksHaveBookmarks: true,
        allNumberingReferencesDefined: true,
        allTableHeadersMarked: true
      },
      warnings: #[]
    }
  }
];

const PowerPointLowerVectors = #[
  #{
    name: "powerpoint_heading_slides-lower",
    stage: "callable",
    input: #{ markdown: "# Roadmap\n\nIntro text.\n\n---\n\n## Details\n\nMore text.\n" },
    expected: #{
      slides: #[
        #{
          title: "Roadmap",
          elements: #[#{ kind: "text_box", text: "Intro text." }]
        },
        #{
          title: "Details",
          elements: #[#{ kind: "text_box", text: "More text." }]
        }
      ]
    }
  },
  #{
    name: "powerpoint_list_notes-lower",
    stage: "callable",
    input: #{ markdown: "# Agenda\n\n- Why\n- How\n\n```notes\nOpen with the problem.\nPreview the demo.\n```\n" },
    expected: #{
      slides: #[
        #{
          title: "Agenda",
          elements: #[#{ kind: "text_box", text: "", bullets: #["Why", "How"] }],
          speakerNotes: #["Open with the problem.", "Preview the demo."]
        }
      ]
    }
  },
  #{
    name: "powerpoint_link_image-lower",
    stage: "callable",
    input: #{ markdown: "# References\n\n[Guide](https://example.com/guide)\n\n![Architecture](arch.png \"Diagram\")\n" },
    expected: #{
      slides: #[
        #{
          title: "References",
          elements: #[
            #{ kind: "text_box", text: "Guide", hyperlink: "https://example.com/guide" },
            #{ kind: "text_box", text: "Architecture" }
          ]
        }
      ]
    }
  },
  #{
    name: "powerpoint_front_matter_metadata-lower",
    stage: "callable",
    input: #{ markdown: "---\ntitle: Demo deck\ncreator: Quilldown\nkeywords: ai, office\n---\n# Hello\n\nBody.\n" },
    expected: #{
      metadata: #{ title: "Demo deck", creator: "Quilldown", keywords: #["ai", "office"] },
      slides: #[
        #{
          title: "Hello",
          elements: #[#{ kind: "text_box", text: "Body." }]
        }
      ]
    }
  },
  #{
    name: "powerpoint_no_heading_single_slide-lower",
    stage: "callable",
    input: #{ markdown: "Lead paragraph.\n\nSecond paragraph with `code`.\n" },
    expected: #{
      slides: #[
        #{
          elements: #[
            #{ kind: "text_box", text: "Lead paragraph." },
            #{ kind: "text_box", text: "Second paragraph with code." }
          ]
        }
      ]
    }
  },
  #{
    name: "powerpoint_consecutive_headings_split-lower",
    stage: "callable",
    input: #{ markdown: "# One\n\n## Two\n\n### Detail\n" },
    expected: #{
      slides: #[
        #{ title: "One", elements: #[] },
        #{
          title: "Two",
          elements: #[#{ kind: "text_box", text: "Detail" }]
        }
      ]
    }
  },
  #{
    name: "powerpoint_empty_heading_splits_consistently-lower",
    stage: "callable",
    input: #{ markdown: "#\n\n# Two\n" },
    expected: #{
      slides: #[
        #{ title: "", elements: #[] },
        #{ title: "Two", elements: #[] }
      ]
    }
  },
  #{
    name: "powerpoint_mixed_link_flattens_without_hyperlink-lower",
    stage: "callable",
    input: #{ markdown: "# Links\n\nRead the [guide](https://example.com/guide) today.\n" },
    expected: #{
      slides: #[
        #{
          title: "Links",
          elements: #[#{ kind: "text_box", text: "Read the guide today." }]
        }
      ]
    }
  },
  #{
    name: "powerpoint_notes_blank_lines_filtered-lower",
    stage: "callable",
    input: #{ markdown: "# Notes\n\n```speaker-notes\nFirst note.\n\nSecond note.\n```\n" },
    expected: #{
      slides: #[
        #{
          title: "Notes",
          elements: #[],
          speakerNotes: #["First note.", "Second note."]
        }
      ]
    }
  },
  #{
    name: "powerpoint_inline_image_alt_flattens-lower",
    stage: "callable",
    input: #{ markdown: "# Images\n\nDiagram: ![system map](map.png) in context.\n" },
    expected: #{
      slides: #[
        #{
          title: "Images",
          elements: #[#{ kind: "text_box", text: "Diagram: system map in context." }]
        }
      ]
    }
  },
  #{
    name: "powerpoint_alert_flattens_to_text-lower",
    stage: "callable",
    input: #{ markdown: "# Status\n\n> [!WARNING] Heads up\n> Check the release notes.\n" },
    expected: #{
      slides: #[
        #{
          title: "Status",
          elements: #[#{ kind: "text_box", text: "Heads up\nCheck the release notes." }]
        }
      ]
    }
  }
];

const PowerPointEmitVectors = #[
  #{
    name: "powerpoint_title_slide_manifest-emit",
    stage: "callable",
    input: #{
      presentation: #{
        metadata: #{ title: "Demo deck", creator: "Quilldown" },
        slides: #[
          #{ title: "Hello", elements: #[#{ kind: "text_box", text: "Welcome" }] }
        ]
      }
    },
    expected: #{
      stats: #{ slides: 1, textBoxes: 1, bulletItems: 0, images: 0, links: 0, speakerNotes: 0, warnings: #[] },
      manifest: #{
        kind: "pptx",
        parts: #[
          #{ name: "/docProps/core.xml", contentType: "application/vnd.openxmlformats-package.core-properties+xml", role: "core-properties" },
          #{ name: "/ppt/presentation.xml", contentType: "application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml", role: "presentation" },
          #{ name: "/ppt/slides/slide1.xml", contentType: "application/vnd.openxmlformats-officedocument.presentationml.slide+xml", role: "slide" }
        ],
        relationships: #[
          #{
            relationships: #[
              #{ type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument", target: "ppt/presentation.xml" },
              #{ type: "http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties", target: "docProps/core.xml" }
            ]
          },
          #{
            sourcePart: "/ppt/presentation.xml",
            relationships: #[
              #{ type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide", target: "slides/slide1.xml" }
            ]
          }
        ],
        contentTypes: #[
          #{ extension: "rels", contentType: "application/vnd.openxmlformats-package.relationships+xml" },
          #{ extension: "xml", contentType: "application/xml" },
          #{ partName: "/docProps/core.xml", contentType: "application/vnd.openxmlformats-package.core-properties+xml" },
          #{ partName: "/ppt/presentation.xml", contentType: "application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml" },
          #{ partName: "/ppt/slides/slide1.xml", contentType: "application/vnd.openxmlformats-officedocument.presentationml.slide+xml" }
        ],
        coreProperties: #{ title: "Demo deck", creator: "Quilldown" }
      },
      invariants: #{
        allRelationshipIdsResolve: true,
        allContentTypesDeclared: true,
        noDuplicateRelationshipIds: true,
        noOrphanRequiredParts: true,
        allSlidesReachable: true,
        allMediaPartsReachable: true,
        allHyperlinksHaveRelationships: true
      },
      warnings: #[]
    }
  },
  #{
    name: "powerpoint_bullets_notes_manifest-emit",
    stage: "callable",
    input: #{
      presentation: #{
        slides: #[
          #{
            title: "Agenda",
            elements: #[#{ kind: "text_box", text: "Topics", bullets: #["Why", "How"] }],
            speakerNotes: #["Open with the problem.", "Preview the demo."]
          }
        ]
      }
    },
    expected: #{
      stats: #{ slides: 1, textBoxes: 1, bulletItems: 2, images: 0, links: 0, speakerNotes: 2, warnings: #[] },
      manifest: #{
        kind: "pptx",
        parts: #[
          #{ name: "/docProps/core.xml", contentType: "application/vnd.openxmlformats-package.core-properties+xml", role: "core-properties" },
          #{ name: "/ppt/notesSlides/notesSlide1.xml", contentType: "application/vnd.openxmlformats-officedocument.presentationml.notesSlide+xml", role: "notes" },
          #{ name: "/ppt/presentation.xml", contentType: "application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml", role: "presentation" },
          #{ name: "/ppt/slides/slide1.xml", contentType: "application/vnd.openxmlformats-officedocument.presentationml.slide+xml", role: "slide" }
        ],
        relationships: #[
          #{
            relationships: #[
              #{ type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument", target: "ppt/presentation.xml" },
              #{ type: "http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties", target: "docProps/core.xml" }
            ]
          },
          #{
            sourcePart: "/ppt/presentation.xml",
            relationships: #[
              #{ type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide", target: "slides/slide1.xml" }
            ]
          },
          #{
            sourcePart: "/ppt/slides/slide1.xml",
            relationships: #[
              #{ type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/notesSlide", target: "../notesSlides/notesSlide1.xml" }
            ]
          }
        ],
        contentTypes: #[
          #{ extension: "rels", contentType: "application/vnd.openxmlformats-package.relationships+xml" },
          #{ extension: "xml", contentType: "application/xml" },
          #{ partName: "/docProps/core.xml", contentType: "application/vnd.openxmlformats-package.core-properties+xml" },
          #{ partName: "/ppt/notesSlides/notesSlide1.xml", contentType: "application/vnd.openxmlformats-officedocument.presentationml.notesSlide+xml" },
          #{ partName: "/ppt/presentation.xml", contentType: "application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml" },
          #{ partName: "/ppt/slides/slide1.xml", contentType: "application/vnd.openxmlformats-officedocument.presentationml.slide+xml" }
        ]
      },
      invariants: #{
        allRelationshipIdsResolve: true,
        allContentTypesDeclared: true,
        noDuplicateRelationshipIds: true,
        noOrphanRequiredParts: true,
        allSlidesReachable: true,
        allMediaPartsReachable: true,
        allHyperlinksHaveRelationships: true
      },
      warnings: #[]
    }
  },
  #{
    name: "powerpoint_hyperlink_relationship-emit",
    stage: "callable",
    input: #{
      presentation: #{ slides: #[#{ elements: #[#{ kind: "text_box", text: "Guide", hyperlink: "https://example.com/guide" }] }] }
    },
    expected: #{
      stats: #{ slides: 1, textBoxes: 1, bulletItems: 0, images: 0, links: 1, speakerNotes: 0, warnings: #[] },
      manifest: #{
        kind: "pptx",
        parts: #[
          #{ name: "/docProps/core.xml", contentType: "application/vnd.openxmlformats-package.core-properties+xml", role: "core-properties" },
          #{ name: "/ppt/presentation.xml", contentType: "application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml", role: "presentation" },
          #{ name: "/ppt/slides/slide1.xml", contentType: "application/vnd.openxmlformats-officedocument.presentationml.slide+xml", role: "slide" }
        ],
        relationships: #[
          #{
            relationships: #[
              #{ type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument", target: "ppt/presentation.xml" },
              #{ type: "http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties", target: "docProps/core.xml" }
            ]
          },
          #{
            sourcePart: "/ppt/presentation.xml",
            relationships: #[
              #{ type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide", target: "slides/slide1.xml" }
            ]
          },
          #{
            sourcePart: "/ppt/slides/slide1.xml",
            relationships: #[
              #{ type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink", target: "https://example.com/guide", targetMode: "External" }
            ]
          }
        ],
        contentTypes: #[
          #{ extension: "rels", contentType: "application/vnd.openxmlformats-package.relationships+xml" },
          #{ extension: "xml", contentType: "application/xml" },
          #{ partName: "/docProps/core.xml", contentType: "application/vnd.openxmlformats-package.core-properties+xml" },
          #{ partName: "/ppt/presentation.xml", contentType: "application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml" },
          #{ partName: "/ppt/slides/slide1.xml", contentType: "application/vnd.openxmlformats-officedocument.presentationml.slide+xml" }
        ]
      },
      invariants: #{
        allRelationshipIdsResolve: true,
        allContentTypesDeclared: true,
        noDuplicateRelationshipIds: true,
        noOrphanRequiredParts: true,
        allSlidesReachable: true,
        allMediaPartsReachable: true,
        allHyperlinksHaveRelationships: true
      },
      warnings: #[]
    }
  },
  #{
    name: "powerpoint_two_hyperlinks_relationships-emit",
    stage: "callable",
    input: #{
      presentation: #{
        slides: #[
          #{
            elements: #[
              #{ kind: "text_box", text: "One", hyperlink: "https://example.com/one" },
              #{ kind: "text_box", text: "Two", hyperlink: "https://example.com/two" }
            ]
          }
        ]
      }
    },
    expected: #{
      stats: #{ slides: 1, textBoxes: 2, bulletItems: 0, images: 0, links: 2, speakerNotes: 0, warnings: #[] },
      manifest: #{
        kind: "pptx",
        parts: #[
          #{ name: "/docProps/core.xml", contentType: "application/vnd.openxmlformats-package.core-properties+xml", role: "core-properties" },
          #{ name: "/ppt/presentation.xml", contentType: "application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml", role: "presentation" },
          #{ name: "/ppt/slides/slide1.xml", contentType: "application/vnd.openxmlformats-officedocument.presentationml.slide+xml", role: "slide" }
        ],
        relationships: #[
          #{
            relationships: #[
              #{ type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument", target: "ppt/presentation.xml" },
              #{ type: "http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties", target: "docProps/core.xml" }
            ]
          },
          #{
            sourcePart: "/ppt/presentation.xml",
            relationships: #[
              #{ type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide", target: "slides/slide1.xml" }
            ]
          },
          #{
            sourcePart: "/ppt/slides/slide1.xml",
            relationships: #[
              #{ type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink", target: "https://example.com/one", targetMode: "External" },
              #{ type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink", target: "https://example.com/two", targetMode: "External" }
            ]
          }
        ],
        contentTypes: #[
          #{ extension: "rels", contentType: "application/vnd.openxmlformats-package.relationships+xml" },
          #{ extension: "xml", contentType: "application/xml" },
          #{ partName: "/docProps/core.xml", contentType: "application/vnd.openxmlformats-package.core-properties+xml" },
          #{ partName: "/ppt/presentation.xml", contentType: "application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml" },
          #{ partName: "/ppt/slides/slide1.xml", contentType: "application/vnd.openxmlformats-officedocument.presentationml.slide+xml" }
        ]
      },
      invariants: #{
        allRelationshipIdsResolve: true,
        allContentTypesDeclared: true,
        noDuplicateRelationshipIds: true,
        noOrphanRequiredParts: true,
        allSlidesReachable: true,
        allMediaPartsReachable: true,
        allHyperlinksHaveRelationships: true
      },
      warnings: #[]
    }
  },
  #{
    name: "powerpoint_multi_slide_hyperlink_scoped_relationships-emit",
    stage: "callable",
    input: #{
      presentation: #{
        slides: #[
          #{ elements: #[#{ kind: "text_box", text: "Guide", hyperlink: "https://example.com/guide" }] },
          #{
            title: "Wrap",
            elements: #[#{ kind: "text_box", text: "Questions" }],
            speakerNotes: #["Close with next steps."]
          }
        ]
      }
    },
    expected: #{
      stats: #{ slides: 2, textBoxes: 2, bulletItems: 0, images: 0, links: 1, speakerNotes: 1, warnings: #[] },
      manifest: #{
        kind: "pptx",
        parts: #[
          #{ name: "/docProps/core.xml", contentType: "application/vnd.openxmlformats-package.core-properties+xml", role: "core-properties" },
          #{ name: "/ppt/notesSlides/notesSlide2.xml", contentType: "application/vnd.openxmlformats-officedocument.presentationml.notesSlide+xml", role: "notes" },
          #{ name: "/ppt/presentation.xml", contentType: "application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml", role: "presentation" },
          #{ name: "/ppt/slides/slide1.xml", contentType: "application/vnd.openxmlformats-officedocument.presentationml.slide+xml", role: "slide" },
          #{ name: "/ppt/slides/slide2.xml", contentType: "application/vnd.openxmlformats-officedocument.presentationml.slide+xml", role: "slide" }
        ],
        relationships: #[
          #{
            relationships: #[
              #{ type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument", target: "ppt/presentation.xml" },
              #{ type: "http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties", target: "docProps/core.xml" }
            ]
          },
          #{
            sourcePart: "/ppt/presentation.xml",
            relationships: #[
              #{ type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide", target: "slides/slide1.xml" },
              #{ type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide", target: "slides/slide2.xml" }
            ]
          },
          #{
            sourcePart: "/ppt/slides/slide1.xml",
            relationships: #[
              #{ type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink", target: "https://example.com/guide", targetMode: "External" }
            ]
          },
          #{
            sourcePart: "/ppt/slides/slide2.xml",
            relationships: #[
              #{ type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/notesSlide", target: "../notesSlides/notesSlide2.xml" }
            ]
          }
        ],
        contentTypes: #[
          #{ extension: "rels", contentType: "application/vnd.openxmlformats-package.relationships+xml" },
          #{ extension: "xml", contentType: "application/xml" },
          #{ partName: "/docProps/core.xml", contentType: "application/vnd.openxmlformats-package.core-properties+xml" },
          #{ partName: "/ppt/notesSlides/notesSlide2.xml", contentType: "application/vnd.openxmlformats-officedocument.presentationml.notesSlide+xml" },
          #{ partName: "/ppt/presentation.xml", contentType: "application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml" },
          #{ partName: "/ppt/slides/slide1.xml", contentType: "application/vnd.openxmlformats-officedocument.presentationml.slide+xml" },
          #{ partName: "/ppt/slides/slide2.xml", contentType: "application/vnd.openxmlformats-officedocument.presentationml.slide+xml" }
        ]
      },
      invariants: #{
        allRelationshipIdsResolve: true,
        allContentTypesDeclared: true,
        noDuplicateRelationshipIds: true,
        noOrphanRequiredParts: true,
        allSlidesReachable: true,
        allMediaPartsReachable: true,
        allHyperlinksHaveRelationships: true
      },
      warnings: #[]
    }
  },
  #{
    name: "powerpoint_slide_two_hyperlink_scoped_relationships-emit",
    stage: "callable",
    input: #{
      presentation: #{
        slides: #[
          #{
            title: "Intro",
            elements: #[#{ kind: "text_box", text: "Set context" }],
            speakerNotes: #["Open with context."]
          },
          #{ elements: #[#{ kind: "text_box", text: "Guide", hyperlink: "https://example.com/guide" }] }
        ]
      }
    },
    expected: #{
      stats: #{ slides: 2, textBoxes: 2, bulletItems: 0, images: 0, links: 1, speakerNotes: 1, warnings: #[] },
      manifest: #{
        kind: "pptx",
        parts: #[
          #{ name: "/docProps/core.xml", contentType: "application/vnd.openxmlformats-package.core-properties+xml", role: "core-properties" },
          #{ name: "/ppt/notesSlides/notesSlide1.xml", contentType: "application/vnd.openxmlformats-officedocument.presentationml.notesSlide+xml", role: "notes" },
          #{ name: "/ppt/presentation.xml", contentType: "application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml", role: "presentation" },
          #{ name: "/ppt/slides/slide1.xml", contentType: "application/vnd.openxmlformats-officedocument.presentationml.slide+xml", role: "slide" },
          #{ name: "/ppt/slides/slide2.xml", contentType: "application/vnd.openxmlformats-officedocument.presentationml.slide+xml", role: "slide" }
        ],
        relationships: #[
          #{
            relationships: #[
              #{ type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument", target: "ppt/presentation.xml" },
              #{ type: "http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties", target: "docProps/core.xml" }
            ]
          },
          #{
            sourcePart: "/ppt/presentation.xml",
            relationships: #[
              #{ type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide", target: "slides/slide1.xml" },
              #{ type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide", target: "slides/slide2.xml" }
            ]
          },
          #{
            sourcePart: "/ppt/slides/slide1.xml",
            relationships: #[
              #{ type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/notesSlide", target: "../notesSlides/notesSlide1.xml" }
            ]
          },
          #{
            sourcePart: "/ppt/slides/slide2.xml",
            relationships: #[
              #{ type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink", target: "https://example.com/guide", targetMode: "External" }
            ]
          }
        ],
        contentTypes: #[
          #{ extension: "rels", contentType: "application/vnd.openxmlformats-package.relationships+xml" },
          #{ extension: "xml", contentType: "application/xml" },
          #{ partName: "/docProps/core.xml", contentType: "application/vnd.openxmlformats-package.core-properties+xml" },
          #{ partName: "/ppt/notesSlides/notesSlide1.xml", contentType: "application/vnd.openxmlformats-officedocument.presentationml.notesSlide+xml" },
          #{ partName: "/ppt/presentation.xml", contentType: "application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml" },
          #{ partName: "/ppt/slides/slide1.xml", contentType: "application/vnd.openxmlformats-officedocument.presentationml.slide+xml" },
          #{ partName: "/ppt/slides/slide2.xml", contentType: "application/vnd.openxmlformats-officedocument.presentationml.slide+xml" }
        ]
      },
      invariants: #{
        allRelationshipIdsResolve: true,
        allContentTypesDeclared: true,
        noDuplicateRelationshipIds: true,
        noOrphanRequiredParts: true,
        allSlidesReachable: true,
        allMediaPartsReachable: true,
        allHyperlinksHaveRelationships: true
      },
      warnings: #[]
    }
  },
  #{
    name: "powerpoint_image_fallback_media_warning-emit",
    stage: "callable",
    input: #{ presentation: #{ slides: #[#{ elements: #[#{ kind: "image", src: "cat.png", alt: "a cat" }] }] } },
    expected: #{
      stats: #{ slides: 1, textBoxes: 0, bulletItems: 0, images: 1, links: 0, speakerNotes: 0, warnings: #[] },
      manifest: #{
        kind: "pptx",
        parts: #[
          #{ name: "/docProps/core.xml", contentType: "application/vnd.openxmlformats-package.core-properties+xml", role: "core-properties" },
          #{ name: "/ppt/presentation.xml", contentType: "application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml", role: "presentation" },
          #{ name: "/ppt/slides/slide1.xml", contentType: "application/vnd.openxmlformats-officedocument.presentationml.slide+xml", role: "slide" }
        ],
        relationships: #[
          #{
            relationships: #[
              #{ type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument", target: "ppt/presentation.xml" },
              #{ type: "http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties", target: "docProps/core.xml" }
            ]
          },
          #{
            sourcePart: "/ppt/presentation.xml",
            relationships: #[
              #{ type: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide", target: "slides/slide1.xml" }
            ]
          }
        ],
        contentTypes: #[
          #{ extension: "rels", contentType: "application/vnd.openxmlformats-package.relationships+xml" },
          #{ extension: "xml", contentType: "application/xml" },
          #{ partName: "/docProps/core.xml", contentType: "application/vnd.openxmlformats-package.core-properties+xml" },
          #{ partName: "/ppt/presentation.xml", contentType: "application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml" },
          #{ partName: "/ppt/slides/slide1.xml", contentType: "application/vnd.openxmlformats-officedocument.presentationml.slide+xml" }
        ]
      },
      invariants: #{
        allRelationshipIdsResolve: true,
        allContentTypesDeclared: true,
        noDuplicateRelationshipIds: true,
        noOrphanRequiredParts: true,
        allSlidesReachable: true,
        allMediaPartsReachable: false,
        allHyperlinksHaveRelationships: true
      },
      warnings: #[#{ code: "powerpoint.media-unreachable", severity: "warning", message: "PowerPoint invariant failed: allMediaPartsReachable.", path: "invariants.allMediaPartsReachable" }]
    }
  },
  #{
    name: "powerpoint_image_strict-emit",
    stage: "callable",
    input: #{
      presentation: #{ slides: #[#{ elements: #[#{ kind: "image", src: "cat.png", alt: "a cat" }] }] },
      options: #{ strict: true }
    },
    expectedError: #{ message: "PowerPoint invariant failed: allMediaPartsReachable." }
  },
  #{
    name: "powerpoint_invalid_empty_slides-emit",
    stage: "callable",
    input: #{ presentation: #{ slides: #[] } },
    expectedError: #{ message: "slides must contain at least one slide" }
  },
  #{
    name: "powerpoint_invalid_element_kind-emit",
    stage: "callable",
    input: #{ presentation: #{ slides: #[#{ elements: #[#{ kind: "chart" }] }] } },
    expectedError: #{ message: "slides[0].elements[0] has unsupported element kind 'chart'" }
  },
  #{
    name: "powerpoint_invalid_empty_hyperlink-emit",
    stage: "callable",
    input: #{ presentation: #{ slides: #[#{ elements: #[#{ kind: "text_box", text: "Guide", hyperlink: "" }] }] } },
    expectedError: #{ message: "slides[0].elements[0].hyperlink must not be empty" }
  },
  #{
    name: "powerpoint_invalid_missing_text-emit",
    stage: "callable",
    input: #{ presentation: #{ slides: #[#{ elements: #[#{ kind: "text_box" }] }] } },
    expectedError: #{ message: "slides[0].elements[0].text is required" }
  },
  #{
    name: "powerpoint_invalid_bullets_shape-emit",
    stage: "callable",
    input: #{ presentation: #{ slides: #[#{ elements: #[#{ kind: "text_box", text: "Agenda", bullets: "not-array" }] }] } },
    expectedError: #{ message: "slides[0].elements[0].bullets must be an array" }
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
  },
  #{
    name: "link_in_heading-emit",
    stage: "callable",
    input: #{
      doc: #{
        blocks: #[
          #{
            content: #[
              #{
                kind: "text",
                data: "see "
              },
              #{
                kind: "link",
                href: "#a",
                content: #[
                  #{
                    kind: "text",
                    data: "x"
                  }
                ]
              }
            ],
            kind: "heading",
            level: 2
          }
        ]
      }
    },
    expected: #{
      headings: 1,
      paragraphs: 0,
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
    name: "link_in_table_cells-emit",
    stage: "callable",
    input: #{
      doc: #{
        blocks: #[
          #{
            kind: "table",
            align: #[
              "left"
            ],
            head: #{
              cells: #[
                #{
                  content: #[
                    #{
                      kind: "link",
                      href: "#h",
                      content: #[
                        #{
                          kind: "text",
                          data: "H"
                        }
                      ]
                    }
                  ]
                }
              ]
            },
            rows: #[
              #{
                cells: #[
                  #{
                    content: #[
                      #{
                        kind: "link",
                        href: "#r",
                        content: #[
                          #{
                            kind: "text",
                            data: "R"
                          }
                        ]
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
    expected: #{
      headings: 0,
      paragraphs: 0,
      codeBlocks: 0,
      blockQuotes: 0,
      lists: 0,
      listItems: 0,
      tables: 1,
      links: 2,
      thematicBreaks: 0,
      warnings: #[]
    }
  },
  #{
    name: "link_nested_in_strong-emit",
    stage: "callable",
    input: #{
      doc: #{
        blocks: #[
          #{
            content: #[
              #{
                kind: "strong",
                data: #[
                  #{
                    kind: "link",
                    href: "#a",
                    content: #[
                      #{
                        kind: "text",
                        data: "y"
                      }
                    ]
                  }
                ]
              }
            ],
            kind: "paragraph"
          }
        ]
      }
    },
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
    name: "contract_brief-emit",
    stage: "callable",
    input: #{
      doc: #{
        blocks: #[
          #{
            kind: "heading",
            level: 1,
            content: #[
              #{
                kind: "text",
                data: "Contract Brief"
              }
            ]
          },
          #{
            kind: "heading",
            level: 2,
            content: #[
              #{
                kind: "text",
                data: "Summary"
              }
            ]
          },
          #{
            kind: "list",
            ordered: false,
            start: 1,
            items: #[
              #{
                kind: "list_item",
                blocks: #[
                  #{
                    kind: "paragraph",
                    content: #[
                      #{
                        kind: "text",
                        data: "Supplier: Aster Ridge"
                      }
                    ]
                  }
                ]
              },
              #{
                kind: "list_item",
                blocks: #[
                  #{
                    kind: "paragraph",
                    content: #[
                      #{
                        kind: "text",
                        data: "Monthly minimum: USD 125,000"
                      }
                    ]
                  }
                ]
              }
            ]
          },
          #{
            kind: "heading",
            level: 2,
            content: #[
              #{
                kind: "text",
                data: "Notes"
              }
            ]
          },
          #{
            kind: "paragraph",
            content: #[
              #{
                kind: "text",
                data: "Generated from Markdown with quilldown."
              }
            ]
          }
        ]
      }
    },
    expected: #{
      headings: 3,
      paragraphs: 3,
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
    name: "invalid_heading_level_zero-emit",
    stage: "callable",
    input: #{ doc: #{ blocks: #[#{ kind: "heading", level: 0, content: #[] }] } },
    expectedError: #{ message: "blocks[0].level must be between 1 and 6" }
  },
  #{
    name: "invalid_top_level_list_item-emit",
    stage: "callable",
    input: #{ doc: #{ blocks: #[#{ kind: "list_item", blocks: #[] }] } },
    expectedError: #{ message: "blocks[0] list_item is only valid inside a list" }
  },
  #{
    name: "invalid_list_item_shape-emit",
    stage: "callable",
    input: #{ doc: #{ blocks: #[#{ kind: "list", ordered: false, start: 1, items: #[#{ kind: "paragraph", content: #[] }] }] } },
    expectedError: #{ message: "blocks[0].items[0] must be list_item" }
  },
  #{
    name: "invalid_ordered_start_negative-emit",
    stage: "callable",
    input: #{ doc: #{ blocks: #[#{ kind: "list", ordered: true, start: -1, items: #[#{ kind: "list_item", blocks: #[] }] }] } },
    expectedError: #{ message: "blocks[0].start must be greater than or equal to 1" }
  },
  #{
    name: "invalid_nested_heading_level-emit",
    stage: "callable",
    input: #{ doc: #{ blocks: #[#{ kind: "block_quote", blocks: #[#{ kind: "heading", level: 7, content: #[] }] }] } },
    expectedError: #{ message: "blocks[0].blocks[0].level must be between 1 and 6" }
  },
  #{
    name: "invalid_nested_list_item-emit",
    stage: "callable",
    input: #{ doc: #{ blocks: #[#{ kind: "block_quote", blocks: #[#{ kind: "list_item", blocks: #[] }] }] } },
    expectedError: #{ message: "blocks[0].blocks[0] list_item is only valid inside a list" }
  },
  #{
    name: "invalid_ragged_table-emit",
    stage: "callable",
    input: #{
      doc: #{
        blocks: #[
          #{
            kind: "table",
            align: #["left", "right"],
            head: #{ cells: #[#{ content: #[] }, #{ content: #[] }] },
            rows: #[#{ cells: #[#{ content: #[] }] }]
          }
        ]
      }
    },
    expectedError: #{ message: "blocks[0].rows[0] has inconsistent table width" }
  },
  #{
    name: "invalid_table_alignment-emit",
    stage: "callable",
    input: #{
      doc: #{
        blocks: #[
          #{
            kind: "table",
            align: #["justify"],
            head: #{ cells: #[#{ content: #[] }] },
            rows: #[]
          }
        ]
      }
    },
    expectedError: #{ message: "blocks[0].align contains unsupported alignment" }
  },
  #{
    name: "invalid_table_alignment_width-emit",
    stage: "callable",
    input: #{
      doc: #{
        blocks: #[
          #{
            kind: "table",
            align: #["left"],
            head: #{ cells: #[#{ content: #[] }, #{ content: #[] }] },
            rows: #[]
          }
        ]
      }
    },
    expectedError: #{ message: "blocks[0].align length must match table width" }
  },
  #{
    name: "invalid_missing_math_payload-emit",
    stage: "callable",
    input: #{ doc: #{ blocks: #[#{ kind: "paragraph", content: #[#{ kind: "math" }] }] } },
    expectedError: #{ message: "blocks[0].content[0] missing required field latex" }
  }
];

const RenderVectors = #[
  #{
    name: "render_typography",
    stage: "callable",
    input: #{ doc: #{
      blocks: #[
        #{
          content: #[
            #{
              data: "H1",
              kind: "text"
            }
          ],
          kind: "heading",
          level: 1
        },
        #{
          content: #[
            #{
              data: "Body with ",
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
              content: #[
                #{
                  data: "link",
                  kind: "text"
                }
              ],
              href: "https://example.com",
              kind: "link"
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
        },
        #{
          content: #[
            #{
              data: "H2",
              kind: "text"
            }
          ],
          kind: "heading",
          level: 2
        },
        #{
          content: #[
            #{
              data: "H3",
              kind: "text"
            }
          ],
          kind: "heading",
          level: 3
        },
        #{
          content: #[
            #{
              data: "H4",
              kind: "text"
            }
          ],
          kind: "heading",
          level: 4
        },
        #{
          content: #[
            #{
              data: "H5",
              kind: "text"
            }
          ],
          kind: "heading",
          level: 5
        },
        #{
          content: #[
            #{
              data: "H6",
              kind: "text"
            }
          ],
          kind: "heading",
          level: 6
        }
      ]
    } },
    expected: #{
      body: #[
        #{
          align: null,
          borders: null,
          fill: null,
          gapBefore: 360,
          headingLevel: 1,
          indent: #{
            firstLine: 0,
            hanging: 0,
            left: 0,
            right: 0
          },
          keepLines: true,
          keepNext: true,
          kind: "paragraph",
          line: 259,
          lineRule: "auto",
          list: null,
          runs: #[
            #{
              bold: true,
              color: "2F5496",
              font: "Aptos Display",
              italic: false,
              link: null,
              size: 40,
              strike: false,
              text: "H1",
              underline: null,
              vertAlign: null
            }
          ],
          style: "heading1"
        },
        #{
          align: null,
          borders: null,
          fill: null,
          gapBefore: 80,
          headingLevel: null,
          indent: #{
            firstLine: 0,
            hanging: 0,
            left: 0,
            right: 0
          },
          keepLines: false,
          keepNext: false,
          kind: "paragraph",
          line: 259,
          lineRule: "auto",
          list: null,
          runs: #[
            #{
              bold: false,
              color: null,
              font: "Aptos",
              italic: false,
              link: null,
              size: 24,
              strike: false,
              text: "Body with ",
              underline: null,
              vertAlign: null
            },
            #{
              bold: true,
              color: null,
              font: "Aptos",
              italic: false,
              link: null,
              size: 24,
              strike: false,
              text: "bold",
              underline: null,
              vertAlign: null
            },
            #{
              bold: false,
              color: null,
              font: "Aptos",
              italic: false,
              link: null,
              size: 24,
              strike: false,
              text: ", ",
              underline: null,
              vertAlign: null
            },
            #{
              bold: false,
              color: null,
              font: "Aptos",
              italic: true,
              link: null,
              size: 24,
              strike: false,
              text: "italic",
              underline: null,
              vertAlign: null
            },
            #{
              bold: false,
              color: null,
              font: "Aptos",
              italic: false,
              link: null,
              size: 24,
              strike: false,
              text: ", ",
              underline: null,
              vertAlign: null
            },
            #{
              bold: false,
              color: "0563C1",
              font: "Aptos",
              italic: false,
              link: "https://example.com",
              size: 24,
              strike: false,
              text: "link",
              underline: "single",
              vertAlign: null
            },
            #{
              bold: false,
              color: null,
              font: "Aptos",
              italic: false,
              link: null,
              size: 24,
              strike: false,
              text: ", and ",
              underline: null,
              vertAlign: null
            },
            #{
              bold: false,
              color: null,
              font: "Consolas",
              italic: false,
              link: null,
              size: 20,
              strike: false,
              text: "code",
              underline: null,
              vertAlign: null
            },
            #{
              bold: false,
              color: null,
              font: "Aptos",
              italic: false,
              link: null,
              size: 24,
              strike: false,
              text: ".",
              underline: null,
              vertAlign: null
            }
          ],
          style: "normal"
        },
        #{
          align: null,
          borders: null,
          fill: null,
          gapBefore: 360,
          headingLevel: 2,
          indent: #{
            firstLine: 0,
            hanging: 0,
            left: 0,
            right: 0
          },
          keepLines: true,
          keepNext: true,
          kind: "paragraph",
          line: 259,
          lineRule: "auto",
          list: null,
          runs: #[
            #{
              bold: true,
              color: "2F5496",
              font: "Aptos Display",
              italic: false,
              link: null,
              size: 32,
              strike: false,
              text: "H2",
              underline: null,
              vertAlign: null
            }
          ],
          style: "heading2"
        },
        #{
          align: null,
          borders: null,
          fill: null,
          gapBefore: 240,
          headingLevel: 3,
          indent: #{
            firstLine: 0,
            hanging: 0,
            left: 0,
            right: 0
          },
          keepLines: true,
          keepNext: true,
          kind: "paragraph",
          line: 259,
          lineRule: "auto",
          list: null,
          runs: #[
            #{
              bold: true,
              color: "2F5496",
              font: "Aptos Display",
              italic: false,
              link: null,
              size: 28,
              strike: false,
              text: "H3",
              underline: null,
              vertAlign: null
            }
          ],
          style: "heading3"
        },
        #{
          align: null,
          borders: null,
          fill: null,
          gapBefore: 220,
          headingLevel: 4,
          indent: #{
            firstLine: 0,
            hanging: 0,
            left: 0,
            right: 0
          },
          keepLines: true,
          keepNext: true,
          kind: "paragraph",
          line: 259,
          lineRule: "auto",
          list: null,
          runs: #[
            #{
              bold: true,
              color: "2F5496",
              font: "Aptos Display",
              italic: false,
              link: null,
              size: 24,
              strike: false,
              text: "H4",
              underline: null,
              vertAlign: null
            }
          ],
          style: "heading4"
        },
        #{
          align: null,
          borders: null,
          fill: null,
          gapBefore: 200,
          headingLevel: 5,
          indent: #{
            firstLine: 0,
            hanging: 0,
            left: 0,
            right: 0
          },
          keepLines: true,
          keepNext: true,
          kind: "paragraph",
          line: 259,
          lineRule: "auto",
          list: null,
          runs: #[
            #{
              bold: true,
              color: "2F5496",
              font: "Aptos Display",
              italic: false,
              link: null,
              size: 22,
              strike: false,
              text: "H5",
              underline: null,
              vertAlign: null
            }
          ],
          style: "heading5"
        },
        #{
          align: null,
          borders: null,
          fill: null,
          gapBefore: 200,
          headingLevel: 6,
          indent: #{
            firstLine: 0,
            hanging: 0,
            left: 0,
            right: 0
          },
          keepLines: true,
          keepNext: true,
          kind: "paragraph",
          line: 259,
          lineRule: "auto",
          list: null,
          runs: #[
            #{
              bold: true,
              color: "2F5496",
              font: "Aptos Display",
              italic: false,
              link: null,
              size: 20,
              strike: false,
              text: "H6",
              underline: null,
              vertAlign: null
            }
          ],
          style: "heading6"
        }
      ],
      page: #{
        footer: null,
        header: null,
        height: 15840,
        landscape: false,
        margins: #{
          bottom: 1440,
          footer: 720,
          header: 720,
          left: 1440,
          right: 1440,
          top: 1440
        },
        width: 12240
      },
      trailingGap: 80
    }
  },
  #{
    name: "render_tables_tasks",
    stage: "callable",
    input: #{ doc: #{
      blocks: #[
        #{
          items: #[
            #{
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
              kind: "list_item",
              task: true
            },
            #{
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
              kind: "list_item",
              task: false
            }
          ],
          kind: "list",
          ordered: false,
          start: 1
        },
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
                    data: "A",
                    kind: "text"
                  }
                ]
              },
              #{
                content: #[
                  #{
                    data: "B",
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
                      data: "1",
                      kind: "text"
                    }
                  ]
                },
                #{
                  content: #[
                    #{
                      data: "2",
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
      body: #[
        #{
          align: null,
          borders: null,
          fill: null,
          gapBefore: 0,
          headingLevel: null,
          indent: #{
            firstLine: 0,
            hanging: 360,
            left: 720,
            right: 0
          },
          keepLines: false,
          keepNext: false,
          kind: "paragraph",
          line: 259,
          lineRule: "auto",
          list: null,
          runs: #[
            #{
              bold: false,
              color: null,
              font: "Aptos",
              italic: false,
              link: null,
              size: 24,
              strike: false,
              text: "☑\tdone",
              underline: null,
              vertAlign: null
            }
          ],
          style: "normal"
        },
        #{
          align: null,
          borders: null,
          fill: null,
          gapBefore: 0,
          headingLevel: null,
          indent: #{
            firstLine: 0,
            hanging: 360,
            left: 720,
            right: 0
          },
          keepLines: false,
          keepNext: false,
          kind: "paragraph",
          line: 259,
          lineRule: "auto",
          list: null,
          runs: #[
            #{
              bold: false,
              color: null,
              font: "Aptos",
              italic: false,
              link: null,
              size: 24,
              strike: false,
              text: "☐\ttodo",
              underline: null,
              vertAlign: null
            }
          ],
          style: "normal"
        },
        #{
          align: null,
          borders: #{
            bottom: #{
              color: "BFBFBF",
              size: 2,
              space: 0,
              style: "single"
            },
            insideH: #{
              color: "BFBFBF",
              size: 2,
              space: 0,
              style: "single"
            },
            insideV: #{
              color: "BFBFBF",
              size: 2,
              space: 0,
              style: "single"
            },
            left: #{
              color: "BFBFBF",
              size: 2,
              space: 0,
              style: "single"
            },
            right: #{
              color: "BFBFBF",
              size: 2,
              space: 0,
              style: "single"
            },
            top: #{
              color: "BFBFBF",
              size: 2,
              space: 0,
              style: "single"
            }
          },
          cellMargins: #{
            bottom: 40,
            left: 108,
            right: 108,
            top: 40
          },
          columns: #[],
          gapBefore: 160,
          kind: "table",
          rows: #[
            #{
              cells: #[
                #{
                  blocks: #[
                    #{
                      align: null,
                      borders: null,
                      fill: null,
                      gapBefore: 0,
                      headingLevel: null,
                      indent: #{
                        firstLine: 0,
                        hanging: 0,
                        left: 0,
                        right: 0
                      },
                      keepLines: false,
                      keepNext: false,
                      kind: "paragraph",
                      line: 259,
                      lineRule: "auto",
                      list: null,
                      runs: #[
                        #{
                          bold: true,
                          color: null,
                          font: "Aptos",
                          italic: false,
                          link: null,
                          size: 24,
                          strike: false,
                          text: "A",
                          underline: null,
                          vertAlign: null
                        }
                      ],
                      style: "normal"
                    }
                  ],
                  borders: null,
                  fill: "D9D9D9",
                  span: 1,
                  trailingGap: 0,
                  vMerge: null,
                  width: null
                },
                #{
                  blocks: #[
                    #{
                      align: "right",
                      borders: null,
                      fill: null,
                      gapBefore: 0,
                      headingLevel: null,
                      indent: #{
                        firstLine: 0,
                        hanging: 0,
                        left: 0,
                        right: 0
                      },
                      keepLines: false,
                      keepNext: false,
                      kind: "paragraph",
                      line: 259,
                      lineRule: "auto",
                      list: null,
                      runs: #[
                        #{
                          bold: true,
                          color: null,
                          font: "Aptos",
                          italic: false,
                          link: null,
                          size: 24,
                          strike: false,
                          text: "B",
                          underline: null,
                          vertAlign: null
                        }
                      ],
                      style: "normal"
                    }
                  ],
                  borders: null,
                  fill: "D9D9D9",
                  span: 1,
                  trailingGap: 0,
                  vMerge: null,
                  width: null
                }
              ],
              header: true
            },
            #{
              cells: #[
                #{
                  blocks: #[
                    #{
                      align: null,
                      borders: null,
                      fill: null,
                      gapBefore: 0,
                      headingLevel: null,
                      indent: #{
                        firstLine: 0,
                        hanging: 0,
                        left: 0,
                        right: 0
                      },
                      keepLines: false,
                      keepNext: false,
                      kind: "paragraph",
                      line: 259,
                      lineRule: "auto",
                      list: null,
                      runs: #[
                        #{
                          bold: false,
                          color: null,
                          font: "Aptos",
                          italic: false,
                          link: null,
                          size: 24,
                          strike: false,
                          text: "1",
                          underline: null,
                          vertAlign: null
                        }
                      ],
                      style: "normal"
                    }
                  ],
                  borders: null,
                  fill: null,
                  span: 1,
                  trailingGap: 0,
                  vMerge: null,
                  width: null
                },
                #{
                  blocks: #[
                    #{
                      align: "right",
                      borders: null,
                      fill: null,
                      gapBefore: 0,
                      headingLevel: null,
                      indent: #{
                        firstLine: 0,
                        hanging: 0,
                        left: 0,
                        right: 0
                      },
                      keepLines: false,
                      keepNext: false,
                      kind: "paragraph",
                      line: 259,
                      lineRule: "auto",
                      list: null,
                      runs: #[
                        #{
                          bold: false,
                          color: null,
                          font: "Aptos",
                          italic: false,
                          link: null,
                          size: 24,
                          strike: false,
                          text: "2",
                          underline: null,
                          vertAlign: null
                        }
                      ],
                      style: "normal"
                    }
                  ],
                  borders: null,
                  fill: null,
                  span: 1,
                  trailingGap: 0,
                  vMerge: null,
                  width: null
                }
              ],
              header: false
            }
          ],
          width: #{
            type: "dxa",
            w: 9360
          }
        }
      ],
      page: #{
        footer: null,
        header: null,
        height: 15840,
        landscape: false,
        margins: #{
          bottom: 1440,
          footer: 720,
          header: 720,
          left: 1440,
          right: 1440,
          top: 1440
        },
        width: 12240
      },
      trailingGap: 0
    }
  },
  #{
    name: "render_quotes_code_rule",
    stage: "callable",
    input: #{ doc: #{
      blocks: #[
        #{
          blocks: #[
            #{
              content: #[
                #{
                  data: "A quote with ",
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
                  data: " and ",
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
          ],
          kind: "block_quote"
        },
        #{
          code: "x = 1\n",
          kind: "code_block"
        },
        #{
          kind: "thematic_break"
        }
      ]
    } },
    expected: #{
      body: #[
        #{
          align: null,
          borders: #{
            left: #{
              color: "8B949E",
              size: 24,
              space: 12,
              style: "single"
            }
          },
          fill: null,
          gapBefore: 0,
          headingLevel: null,
          indent: #{
            firstLine: 0,
            hanging: 0,
            left: 360,
            right: 0
          },
          keepLines: false,
          keepNext: false,
          kind: "paragraph",
          line: 259,
          lineRule: "auto",
          list: null,
          runs: #[
            #{
              bold: false,
              color: "57606A",
              font: "Aptos",
              italic: false,
              link: null,
              size: 24,
              strike: false,
              text: "A quote with ",
              underline: null,
              vertAlign: null
            },
            #{
              bold: true,
              color: "57606A",
              font: "Aptos",
              italic: false,
              link: null,
              size: 24,
              strike: false,
              text: "bold",
              underline: null,
              vertAlign: null
            },
            #{
              bold: false,
              color: "57606A",
              font: "Aptos",
              italic: false,
              link: null,
              size: 24,
              strike: false,
              text: " and ",
              underline: null,
              vertAlign: null
            },
            #{
              bold: false,
              color: null,
              font: "Consolas",
              italic: false,
              link: null,
              size: 20,
              strike: false,
              text: "code",
              underline: null,
              vertAlign: null
            },
            #{
              bold: false,
              color: "57606A",
              font: "Aptos",
              italic: false,
              link: null,
              size: 24,
              strike: false,
              text: ".",
              underline: null,
              vertAlign: null
            }
          ],
          style: "normal"
        },
        #{
          align: null,
          borders: #{
            bottom: #{
              color: "000000",
              size: 2,
              space: 0,
              style: "single"
            },
            insideH: #{
              color: "000000",
              size: 2,
              space: 0,
              style: "single"
            },
            insideV: #{
              color: "000000",
              size: 2,
              space: 0,
              style: "single"
            },
            left: #{
              color: "000000",
              size: 2,
              space: 0,
              style: "single"
            },
            right: #{
              color: "000000",
              size: 2,
              space: 0,
              style: "single"
            },
            top: #{
              color: "000000",
              size: 2,
              space: 0,
              style: "single"
            }
          },
          cellMargins: #{
            bottom: 80,
            left: 120,
            right: 120,
            top: 80
          },
          columns: #[],
          gapBefore: 320,
          kind: "table",
          rows: #[
            #{
              cells: #[
                #{
                  blocks: #[
                    #{
                      align: null,
                      borders: null,
                      fill: null,
                      gapBefore: 0,
                      headingLevel: null,
                      indent: #{
                        firstLine: 0,
                        hanging: 0,
                        left: 0,
                        right: 0
                      },
                      keepLines: false,
                      keepNext: false,
                      kind: "paragraph",
                      line: 240,
                      lineRule: "auto",
                      list: null,
                      runs: #[
                        #{
                          bold: false,
                          color: null,
                          font: "Consolas",
                          italic: false,
                          link: null,
                          size: 20,
                          strike: false,
                          text: "x = 1",
                          underline: null,
                          vertAlign: null
                        }
                      ],
                      style: "normal"
                    }
                  ],
                  borders: null,
                  fill: "F2F2F2",
                  span: 1,
                  trailingGap: 0,
                  vMerge: null,
                  width: null
                }
              ],
              header: false
            }
          ],
          width: #{
            type: "dxa",
            w: 9360
          }
        },
        #{
          align: null,
          borders: #{
            bottom: #{
              color: "BFBFBF",
              size: 4,
              space: 0,
              style: "single"
            }
          },
          cellMargins: null,
          columns: #[],
          gapBefore: 160,
          kind: "table",
          rows: #[
            #{
              cells: #[
                #{
                  blocks: #[
                    #{
                      align: null,
                      borders: null,
                      fill: null,
                      gapBefore: 0,
                      headingLevel: null,
                      indent: #{
                        firstLine: 0,
                        hanging: 0,
                        left: 0,
                        right: 0
                      },
                      keepLines: false,
                      keepNext: false,
                      kind: "paragraph",
                      line: 259,
                      lineRule: "auto",
                      list: null,
                      runs: #[],
                      style: "normal"
                    }
                  ],
                  borders: null,
                  fill: null,
                  span: 1,
                  trailingGap: 160,
                  vMerge: null,
                  width: null
                }
              ],
              header: false
            }
          ],
          width: #{
            type: "dxa",
            w: 9360
          }
        }
      ],
      page: #{
        footer: null,
        header: null,
        height: 15840,
        landscape: false,
        margins: #{
          bottom: 1440,
          footer: 720,
          header: 720,
          left: 1440,
          right: 1440,
          top: 1440
        },
        width: 12240
      },
      trailingGap: 0
    }
  },
  #{
    name: "render_alert",
    stage: "callable",
    input: #{ doc: #{
      blocks: #[
        #{
          alert_type: "warning",
          blocks: #[
            #{
              content: #[
                #{
                  data: "Review ",
                  kind: "text"
                },
                #{
                  data: #[
                    #{
                      data: "pricing",
                      kind: "text"
                    }
                  ],
                  kind: "strong"
                },
                #{
                  data: " before signing.",
                  kind: "text"
                }
              ],
              kind: "paragraph"
            }
          ],
          kind: "alert",
          title: "Contract risk"
        }
      ]
    } },
    expected: #{
      body: #[
        #{
          align: null,
          borders: #{
            left: #{
              color: "9A6700",
              size: 24,
              space: 0,
              style: "single"
            }
          },
          cellMargins: #{
            bottom: 40,
            left: 108,
            right: 108,
            top: 40
          },
          columns: #[],
          gapBefore: 0,
          kind: "table",
          rows: #[
            #{
              cells: #[
                #{
                  blocks: #[
                    #{
                      align: null,
                      borders: null,
                      fill: null,
                      gapBefore: 0,
                      headingLevel: null,
                      indent: #{
                        firstLine: 0,
                        hanging: 0,
                        left: 0,
                        right: 0
                      },
                      keepLines: false,
                      keepNext: false,
                      kind: "paragraph",
                      line: 259,
                      lineRule: "auto",
                      list: null,
                      runs: #[
                        #{
                          bold: true,
                          color: "9A6700",
                          font: "Aptos",
                          italic: false,
                          link: null,
                          size: 24,
                          strike: false,
                          text: "Contract risk",
                          underline: null,
                          vertAlign: null
                        }
                      ],
                      style: "normal"
                    },
                    #{
                      align: null,
                      borders: null,
                      fill: null,
                      gapBefore: 160,
                      headingLevel: null,
                      indent: #{
                        firstLine: 0,
                        hanging: 0,
                        left: 0,
                        right: 0
                      },
                      keepLines: false,
                      keepNext: false,
                      kind: "paragraph",
                      line: 259,
                      lineRule: "auto",
                      list: null,
                      runs: #[
                        #{
                          bold: false,
                          color: null,
                          font: "Aptos",
                          italic: false,
                          link: null,
                          size: 24,
                          strike: false,
                          text: "Review ",
                          underline: null,
                          vertAlign: null
                        },
                        #{
                          bold: true,
                          color: null,
                          font: "Aptos",
                          italic: false,
                          link: null,
                          size: 24,
                          strike: false,
                          text: "pricing",
                          underline: null,
                          vertAlign: null
                        },
                        #{
                          bold: false,
                          color: null,
                          font: "Aptos",
                          italic: false,
                          link: null,
                          size: 24,
                          strike: false,
                          text: " before signing.",
                          underline: null,
                          vertAlign: null
                        }
                      ],
                      style: "normal"
                    }
                  ],
                  borders: null,
                  fill: "FFF8C5",
                  span: 1,
                  trailingGap: 160,
                  vMerge: null,
                  width: null
                }
              ],
              header: false
            }
          ],
          width: #{
            type: "dxa",
            w: 9360
          }
        }
      ],
      page: #{
        footer: null,
        header: null,
        height: 15840,
        landscape: false,
        margins: #{
          bottom: 1440,
          footer: 720,
          header: 720,
          left: 1440,
          right: 1440,
          top: 1440
        },
        width: 12240
      },
      trailingGap: 0
    }
  },

  #{
    name: "render_options_theme_page_numbers",
    stage: "callable",
    input: #{
      doc: #{
        blocks: #[
          #{
            kind: "heading",
            level: 1,
            content: #[
              #{
                kind: "text",
                data: "GitHub"
              }
            ]
          },
          #{
            kind: "paragraph",
            content: #[
              #{
                kind: "link",
                href: "https://example.com",
                content: #[
                  #{
                    kind: "text",
                    data: "link"
                  }
                ]
              },
              #{
                kind: "text",
                data: " and "
              },
              #{
                kind: "code",
                data: "code"
              }
            ]
          },
          #{
            kind: "code_block",
            code: "x = 1\n"
          }
        ]
      },
      options: #{
        theme: "github",
        page_size: "a4",
        orientation: "landscape",
        margin: 0.5,
        page_numbers: true
      }
    },
    expected: #{
      page: #{
        width: 16838,
        height: 11906,
        landscape: true,
        margins: #{
          top: 720,
          right: 720,
          bottom: 720,
          left: 720,
          header: 720,
          footer: 720
        },
        header: null,
        footer: #[
          #{
            kind: "paragraph",
            style: "normal",
            headingLevel: null,
            list: null,
            align: "center",
            line: 259,
            lineRule: "auto",
            indent: #{
              left: 0,
              right: 0,
              firstLine: 0,
              hanging: 0
            },
            keepNext: false,
            keepLines: false,
            borders: null,
            fill: null,
            runs: #[
              #{
                bold: false,
                italic: false,
                strike: false,
                underline: null,
                font: "Aptos",
                size: 24,
                color: null,
                vertAlign: null,
                link: null,
                text: "Page {PAGE} of {NUMPAGES}"
              }
            ],
            gapBefore: 0
          }
        ]
      },
      body: #[
        #{
          kind: "paragraph",
          style: "heading1",
          headingLevel: 1,
          list: null,
          align: null,
          line: 259,
          lineRule: "auto",
          indent: #{
            left: 0,
            right: 0,
            firstLine: 0,
            hanging: 0
          },
          keepNext: true,
          keepLines: true,
          borders: null,
          fill: null,
          runs: #[
            #{
              bold: true,
              italic: false,
              strike: false,
              underline: null,
              font: "Aptos Display",
              size: 40,
              color: "0969DA",
              vertAlign: null,
              link: null,
              text: "GitHub"
            }
          ],
          gapBefore: 360
        },
        #{
          kind: "paragraph",
          style: "normal",
          headingLevel: null,
          list: null,
          align: null,
          line: 259,
          lineRule: "auto",
          indent: #{
            left: 0,
            right: 0,
            firstLine: 0,
            hanging: 0
          },
          keepNext: false,
          keepLines: false,
          borders: null,
          fill: null,
          runs: #[
            #{
              bold: false,
              italic: false,
              strike: false,
              underline: "single",
              font: "Aptos",
              size: 24,
              color: "0969DA",
              vertAlign: null,
              link: "https://example.com",
              text: "link"
            },
            #{
              bold: false,
              italic: false,
              strike: false,
              underline: null,
              font: "Aptos",
              size: 24,
              color: null,
              vertAlign: null,
              link: null,
              text: " and "
            },
            #{
              bold: false,
              italic: false,
              strike: false,
              underline: null,
              font: "Consolas",
              size: 20,
              color: null,
              vertAlign: null,
              link: null,
              text: "code"
            }
          ],
          gapBefore: 80
        },
        #{
          kind: "table",
          width: #{
            w: 15398,
            type: "dxa"
          },
          columns: #[],
          align: null,
          borders: #{
            top: #{
              style: "single",
              size: 2,
              space: 0,
              color: "000000"
            },
            left: #{
              style: "single",
              size: 2,
              space: 0,
              color: "000000"
            },
            bottom: #{
              style: "single",
              size: 2,
              space: 0,
              color: "000000"
            },
            right: #{
              style: "single",
              size: 2,
              space: 0,
              color: "000000"
            },
            insideH: #{
              style: "single",
              size: 2,
              space: 0,
              color: "000000"
            },
            insideV: #{
              style: "single",
              size: 2,
              space: 0,
              color: "000000"
            }
          },
          cellMargins: #{
            top: 80,
            left: 120,
            bottom: 80,
            right: 120
          },
          rows: #[
            #{
              header: false,
              cells: #[
                #{
                  width: null,
                  span: 1,
                  vMerge: null,
                  fill: "F6F8FA",
                  borders: null,
                  blocks: #[
                    #{
                      kind: "paragraph",
                      style: "normal",
                      headingLevel: null,
                      list: null,
                      align: null,
                      line: 240,
                      lineRule: "auto",
                      indent: #{
                        left: 0,
                        right: 0,
                        firstLine: 0,
                        hanging: 0
                      },
                      keepNext: false,
                      keepLines: false,
                      borders: null,
                      fill: null,
                      runs: #[
                        #{
                          bold: false,
                          italic: false,
                          strike: false,
                          underline: null,
                          font: "Consolas",
                          size: 20,
                          color: null,
                          vertAlign: null,
                          link: null,
                          text: "x = 1"
                        }
                      ],
                      gapBefore: 0
                    }
                  ],
                  trailingGap: 0
                }
              ]
            }
          ],
          gapBefore: 160
        }
      ],
      trailingGap: 0
    }
  },

  #{
    name: "render_option_theme_solarized",
    stage: "callable",
    input: #{
      doc: #{
        blocks: #[
          #{
            content: #[
              #{
                data: "Solarized",
                kind: "text"
              }
            ],
            kind: "heading",
            level: 1
          },
          #{
            content: #[
              #{
                content: #[
                  #{
                    data: "link",
                    kind: "text"
                  }
                ],
                href: "https://example.com",
                kind: "link"
              },
              #{
                data: " and ",
                kind: "text"
              },
              #{
                data: "code",
                kind: "code"
              }
            ],
            kind: "paragraph"
          },
          #{
            code: "x = 1\n",
            kind: "code_block"
          }
        ]
      },
      options: #{
        theme: "solarized"
      }
    },
    expected: #{
      page: #{
        width: 12240,
        height: 15840,
        landscape: false,
        margins: #{
          top: 1440,
          right: 1440,
          bottom: 1440,
          left: 1440,
          header: 720,
          footer: 720
        },
        header: null,
        footer: null
      },
      body: #[
        #{
          kind: "paragraph",
          style: "heading1",
          headingLevel: 1,
          list: null,
          align: null,
          line: 259,
          lineRule: "auto",
          indent: #{
            left: 0,
            right: 0,
            firstLine: 0,
            hanging: 0
          },
          keepNext: true,
          keepLines: true,
          borders: null,
          fill: null,
          runs: #[
            #{
              bold: true,
              italic: false,
              strike: false,
              underline: null,
              font: "Aptos Display",
              size: 40,
              color: "268BD2",
              vertAlign: null,
              link: null,
              text: "Solarized"
            }
          ],
          gapBefore: 360
        },
        #{
          kind: "paragraph",
          style: "normal",
          headingLevel: null,
          list: null,
          align: null,
          line: 259,
          lineRule: "auto",
          indent: #{
            left: 0,
            right: 0,
            firstLine: 0,
            hanging: 0
          },
          keepNext: false,
          keepLines: false,
          borders: null,
          fill: null,
          runs: #[
            #{
              bold: false,
              italic: false,
              strike: false,
              underline: "single",
              font: "Aptos",
              size: 24,
              color: "268BD2",
              vertAlign: null,
              link: "https://example.com",
              text: "link"
            },
            #{
              bold: false,
              italic: false,
              strike: false,
              underline: null,
              font: "Aptos",
              size: 24,
              color: null,
              vertAlign: null,
              link: null,
              text: " and "
            },
            #{
              bold: false,
              italic: false,
              strike: false,
              underline: null,
              font: "Consolas",
              size: 20,
              color: null,
              vertAlign: null,
              link: null,
              text: "code"
            }
          ],
          gapBefore: 80
        },
        #{
          kind: "table",
          width: #{
            w: 9360,
            type: "dxa"
          },
          columns: #[],
          align: null,
          borders: #{
            top: #{
              style: "single",
              size: 2,
              space: 0,
              color: "000000"
            },
            left: #{
              style: "single",
              size: 2,
              space: 0,
              color: "000000"
            },
            bottom: #{
              style: "single",
              size: 2,
              space: 0,
              color: "000000"
            },
            right: #{
              style: "single",
              size: 2,
              space: 0,
              color: "000000"
            },
            insideH: #{
              style: "single",
              size: 2,
              space: 0,
              color: "000000"
            },
            insideV: #{
              style: "single",
              size: 2,
              space: 0,
              color: "000000"
            }
          },
          cellMargins: #{
            top: 80,
            left: 120,
            bottom: 80,
            right: 120
          },
          rows: #[
            #{
              header: false,
              cells: #[
                #{
                  width: null,
                  span: 1,
                  vMerge: null,
                  fill: "FDF6E3",
                  borders: null,
                  blocks: #[
                    #{
                      kind: "paragraph",
                      style: "normal",
                      headingLevel: null,
                      list: null,
                      align: null,
                      line: 240,
                      lineRule: "auto",
                      indent: #{
                        left: 0,
                        right: 0,
                        firstLine: 0,
                        hanging: 0
                      },
                      keepNext: false,
                      keepLines: false,
                      borders: null,
                      fill: null,
                      runs: #[
                        #{
                          bold: false,
                          italic: false,
                          strike: false,
                          underline: null,
                          font: "Consolas",
                          size: 20,
                          color: null,
                          vertAlign: null,
                          link: null,
                          text: "x = 1"
                        }
                      ],
                      gapBefore: 0
                    }
                  ],
                  trailingGap: 0
                }
              ]
            }
          ],
          gapBefore: 160
        }
      ],
      trailingGap: 0
    }
  },

  #{
    name: "render_option_page_geometry",
    stage: "callable",
    input: #{
      doc: #{
        blocks: #[
          #{
            content: #[
              #{
                data: "Body",
                kind: "text"
              }
            ],
            kind: "paragraph"
          },
          #{
            code: "x = 1\n",
            kind: "code_block"
          }
        ]
      },
      options: #{
        page_size: "legal",
        orientation: "landscape",
        margin: 1.25
      }
    },
    expected: #{
      page: #{
        width: 20160,
        height: 12240,
        landscape: true,
        margins: #{
          top: 1800,
          right: 1800,
          bottom: 1800,
          left: 1800,
          header: 720,
          footer: 720
        },
        header: null,
        footer: null
      },
      body: #[
        #{
          kind: "paragraph",
          style: "normal",
          headingLevel: null,
          list: null,
          align: null,
          line: 259,
          lineRule: "auto",
          indent: #{
            left: 0,
            right: 0,
            firstLine: 0,
            hanging: 0
          },
          keepNext: false,
          keepLines: false,
          borders: null,
          fill: null,
          runs: #[
            #{
              bold: false,
              italic: false,
              strike: false,
              underline: null,
              font: "Aptos",
              size: 24,
              color: null,
              vertAlign: null,
              link: null,
              text: "Body"
            }
          ],
          gapBefore: 0
        },
        #{
          kind: "table",
          width: #{
            w: 16560,
            type: "dxa"
          },
          columns: #[],
          align: null,
          borders: #{
            top: #{
              style: "single",
              size: 2,
              space: 0,
              color: "000000"
            },
            left: #{
              style: "single",
              size: 2,
              space: 0,
              color: "000000"
            },
            bottom: #{
              style: "single",
              size: 2,
              space: 0,
              color: "000000"
            },
            right: #{
              style: "single",
              size: 2,
              space: 0,
              color: "000000"
            },
            insideH: #{
              style: "single",
              size: 2,
              space: 0,
              color: "000000"
            },
            insideV: #{
              style: "single",
              size: 2,
              space: 0,
              color: "000000"
            }
          },
          cellMargins: #{
            top: 80,
            left: 120,
            bottom: 80,
            right: 120
          },
          rows: #[
            #{
              header: false,
              cells: #[
                #{
                  width: null,
                  span: 1,
                  vMerge: null,
                  fill: "F2F2F2",
                  borders: null,
                  blocks: #[
                    #{
                      kind: "paragraph",
                      style: "normal",
                      headingLevel: null,
                      list: null,
                      align: null,
                      line: 240,
                      lineRule: "auto",
                      indent: #{
                        left: 0,
                        right: 0,
                        firstLine: 0,
                        hanging: 0
                      },
                      keepNext: false,
                      keepLines: false,
                      borders: null,
                      fill: null,
                      runs: #[
                        #{
                          bold: false,
                          italic: false,
                          strike: false,
                          underline: null,
                          font: "Consolas",
                          size: 20,
                          color: null,
                          vertAlign: null,
                          link: null,
                          text: "x = 1"
                        }
                      ],
                      gapBefore: 0
                    }
                  ],
                  trailingGap: 0
                }
              ]
            }
          ],
          gapBefore: 160
        }
      ],
      trailingGap: 0
    }
  },

  #{
    name: "render_option_page_numbers",
    stage: "callable",
    input: #{
      doc: #{
        blocks: #[
          #{
            content: #[
              #{
                data: "Body",
                kind: "text"
              }
            ],
            kind: "paragraph"
          }
        ]
      },
      options: #{
        page_numbers: true
      }
    },
    expected: #{
      page: #{
        width: 12240,
        height: 15840,
        landscape: false,
        margins: #{
          top: 1440,
          right: 1440,
          bottom: 1440,
          left: 1440,
          header: 720,
          footer: 720
        },
        header: null,
        footer: #[
          #{
            kind: "paragraph",
            style: "normal",
            headingLevel: null,
            list: null,
            align: "center",
            line: 259,
            lineRule: "auto",
            indent: #{
              left: 0,
              right: 0,
              firstLine: 0,
              hanging: 0
            },
            keepNext: false,
            keepLines: false,
            borders: null,
            fill: null,
            runs: #[
              #{
                bold: false,
                italic: false,
                strike: false,
                underline: null,
                font: "Aptos",
                size: 24,
                color: null,
                vertAlign: null,
                link: null,
                text: "Page {PAGE} of {NUMPAGES}"
              }
            ],
            gapBefore: 0
          }
        ]
      },
      body: #[
        #{
          kind: "paragraph",
          style: "normal",
          headingLevel: null,
          list: null,
          align: null,
          line: 259,
          lineRule: "auto",
          indent: #{
            left: 0,
            right: 0,
            firstLine: 0,
            hanging: 0
          },
          keepNext: false,
          keepLines: false,
          borders: null,
          fill: null,
          runs: #[
            #{
              bold: false,
              italic: false,
              strike: false,
              underline: null,
              font: "Aptos",
              size: 24,
              color: null,
              vertAlign: null,
              link: null,
              text: "Body"
            }
          ],
          gapBefore: 0
        }
      ],
      trailingGap: 160
    }
  },

  #{
    name: "render_option_table_of_contents",
    stage: "callable",
    input: #{
      doc: #{
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
                data: "Body",
                kind: "text"
              }
            ],
            kind: "paragraph"
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
                data: "More",
                kind: "text"
              }
            ],
            kind: "paragraph"
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
          },
          #{
            content: #[
              #{
                data: "End",
                kind: "text"
              }
            ],
            kind: "paragraph"
          },
          #{
            content: #[
              #{
                data: "Four",
                kind: "text"
              }
            ],
            kind: "heading",
            level: 4
          },
          #{
            content: #[
              #{
                data: "Not in TOC",
                kind: "text"
              }
            ],
            kind: "paragraph"
          }
        ]
      },
      options: #{
        table_of_contents: true
      }
    },
    expected: #{
      page: #{
        width: 12240,
        height: 15840,
        landscape: false,
        margins: #{
          top: 1440,
          right: 1440,
          bottom: 1440,
          left: 1440,
          header: 720,
          footer: 720
        },
        header: null,
        footer: null
      },
      body: #[
        #{
          kind: "paragraph",
          style: "normal",
          headingLevel: null,
          list: null,
          align: null,
          line: 259,
          lineRule: "auto",
          indent: #{
            left: 0,
            right: 0,
            firstLine: 0,
            hanging: 0
          },
          keepNext: false,
          keepLines: false,
          borders: null,
          fill: null,
          runs: #[
            #{
              bold: true,
              italic: false,
              strike: false,
              underline: null,
              font: "Aptos",
              size: 28,
              color: null,
              vertAlign: null,
              link: null,
              text: "Contents"
            }
          ],
          gapBefore: 0
        },
        #{
          kind: "paragraph",
          style: "normal",
          headingLevel: null,
          list: null,
          align: null,
          line: 259,
          lineRule: "auto",
          indent: #{
            left: 0,
            right: 0,
            firstLine: 0,
            hanging: 0
          },
          keepNext: false,
          keepLines: false,
          borders: null,
          fill: null,
          runs: #[],
          gapBefore: 160
        },
        #{
          kind: "paragraph",
          style: "normal",
          headingLevel: null,
          list: null,
          align: null,
          line: 259,
          lineRule: "auto",
          indent: #{
            left: 0,
            right: 0,
            firstLine: 0,
            hanging: 0
          },
          keepNext: false,
          keepLines: false,
          borders: null,
          fill: null,
          runs: #[
            #{
              bold: false,
              italic: false,
              strike: false,
              underline: null,
              font: "Aptos",
              size: 24,
              color: null,
              vertAlign: null,
              link: null,
              text: "{TOC \\o \"1-3\" \\h}"
            }
          ],
          gapBefore: 160
        },
        #{
          kind: "paragraph",
          style: "normal",
          headingLevel: null,
          list: null,
          align: null,
          line: 259,
          lineRule: "auto",
          indent: #{
            left: 0,
            right: 0,
            firstLine: 0,
            hanging: 0
          },
          keepNext: false,
          keepLines: false,
          borders: null,
          fill: null,
          runs: #[
            #{
              bold: false,
              italic: false,
              strike: false,
              underline: null,
              font: "Aptos",
              size: 24,
              color: null,
              vertAlign: null,
              link: null,
              text: "\n"
            }
          ],
          gapBefore: 160
        },
        #{
          kind: "paragraph",
          style: "heading1",
          headingLevel: 1,
          list: null,
          align: null,
          line: 259,
          lineRule: "auto",
          indent: #{
            left: 0,
            right: 0,
            firstLine: 0,
            hanging: 0
          },
          keepNext: true,
          keepLines: true,
          borders: null,
          fill: null,
          runs: #[
            #{
              bold: true,
              italic: false,
              strike: false,
              underline: null,
              font: "Aptos Display",
              size: 40,
              color: "2F5496",
              vertAlign: null,
              link: null,
              text: "One"
            }
          ],
          gapBefore: 520
        },
        #{
          kind: "paragraph",
          style: "normal",
          headingLevel: null,
          list: null,
          align: null,
          line: 259,
          lineRule: "auto",
          indent: #{
            left: 0,
            right: 0,
            firstLine: 0,
            hanging: 0
          },
          keepNext: false,
          keepLines: false,
          borders: null,
          fill: null,
          runs: #[
            #{
              bold: false,
              italic: false,
              strike: false,
              underline: null,
              font: "Aptos",
              size: 24,
              color: null,
              vertAlign: null,
              link: null,
              text: "Body"
            }
          ],
          gapBefore: 80
        },
        #{
          kind: "paragraph",
          style: "heading2",
          headingLevel: 2,
          list: null,
          align: null,
          line: 259,
          lineRule: "auto",
          indent: #{
            left: 0,
            right: 0,
            firstLine: 0,
            hanging: 0
          },
          keepNext: true,
          keepLines: true,
          borders: null,
          fill: null,
          runs: #[
            #{
              bold: true,
              italic: false,
              strike: false,
              underline: null,
              font: "Aptos Display",
              size: 32,
              color: "2F5496",
              vertAlign: null,
              link: null,
              text: "Two"
            }
          ],
          gapBefore: 360
        },
        #{
          kind: "paragraph",
          style: "normal",
          headingLevel: null,
          list: null,
          align: null,
          line: 259,
          lineRule: "auto",
          indent: #{
            left: 0,
            right: 0,
            firstLine: 0,
            hanging: 0
          },
          keepNext: false,
          keepLines: false,
          borders: null,
          fill: null,
          runs: #[
            #{
              bold: false,
              italic: false,
              strike: false,
              underline: null,
              font: "Aptos",
              size: 24,
              color: null,
              vertAlign: null,
              link: null,
              text: "More"
            }
          ],
          gapBefore: 80
        },
        #{
          kind: "paragraph",
          style: "heading3",
          headingLevel: 3,
          list: null,
          align: null,
          line: 259,
          lineRule: "auto",
          indent: #{
            left: 0,
            right: 0,
            firstLine: 0,
            hanging: 0
          },
          keepNext: true,
          keepLines: true,
          borders: null,
          fill: null,
          runs: #[
            #{
              bold: true,
              italic: false,
              strike: false,
              underline: null,
              font: "Aptos Display",
              size: 28,
              color: "2F5496",
              vertAlign: null,
              link: null,
              text: "Three"
            }
          ],
          gapBefore: 320
        },
        #{
          kind: "paragraph",
          style: "normal",
          headingLevel: null,
          list: null,
          align: null,
          line: 259,
          lineRule: "auto",
          indent: #{
            left: 0,
            right: 0,
            firstLine: 0,
            hanging: 0
          },
          keepNext: false,
          keepLines: false,
          borders: null,
          fill: null,
          runs: #[
            #{
              bold: false,
              italic: false,
              strike: false,
              underline: null,
              font: "Aptos",
              size: 24,
              color: null,
              vertAlign: null,
              link: null,
              text: "End"
            }
          ],
          gapBefore: 80
        },
        #{
          kind: "paragraph",
          style: "heading4",
          headingLevel: 4,
          list: null,
          align: null,
          line: 259,
          lineRule: "auto",
          indent: #{
            left: 0,
            right: 0,
            firstLine: 0,
            hanging: 0
          },
          keepNext: true,
          keepLines: true,
          borders: null,
          fill: null,
          runs: #[
            #{
              bold: true,
              italic: false,
              strike: false,
              underline: null,
              font: "Aptos Display",
              size: 24,
              color: "2F5496",
              vertAlign: null,
              link: null,
              text: "Four"
            }
          ],
          gapBefore: 300
        },
        #{
          kind: "paragraph",
          style: "normal",
          headingLevel: null,
          list: null,
          align: null,
          line: 259,
          lineRule: "auto",
          indent: #{
            left: 0,
            right: 0,
            firstLine: 0,
            hanging: 0
          },
          keepNext: false,
          keepLines: false,
          borders: null,
          fill: null,
          runs: #[
            #{
              bold: false,
              italic: false,
              strike: false,
              underline: null,
              font: "Aptos",
              size: 24,
              color: null,
              vertAlign: null,
              link: null,
              text: "Not in TOC"
            }
          ],
          gapBefore: 80
        }
      ],
      trailingGap: 160
    }
  },

  #{
      name: "render_option_captions",
      stage: "callable",
      input: #{
        doc: #{
          blocks: #[
            #{
              kind: "paragraph",
              content: #[
                #{
                  kind: "text",
                  data: "See "
                },
                #{
                  kind: "link",
                  href: "#flow",
                  content: #[
                    #{
                      kind: "text",
                      data: "the figure"
                    }
                  ]
                },
                #{
                  kind: "text",
                  data: " and "
                },
                #{
                  kind: "link",
                  href: "#summary",
                  content: #[
                    #{
                      kind: "text",
                      data: "the table"
                    }
                  ]
                },
                #{
                  kind: "text",
                  data: "."
                }
              ]
            },
            #{
              kind: "paragraph",
              content: #[
                #{
                  kind: "text",
                  data: "Figure: A flow diagram {#flow}"
                }
              ]
            },
            #{
              kind: "paragraph",
              content: #[
                #{
                  kind: "text",
                  data: "Table: Summary values {#summary}"
                }
              ]
            },
            #{
              kind: "block_quote",
              blocks: #[
                #{
                  kind: "paragraph",
                  content: #[
                    #{
                      kind: "text",
                      data: "Figure: Quoted {#quoted}"
                    }
                  ]
                }
              ]
            },
            #{
              kind: "paragraph",
              content: #[
                #{
                  kind: "text",
                  data: "Jump to "
                },
                #{
                  kind: "link",
                  href: "#quoted",
                  content: #[
                    #{
                      kind: "text",
                      data: "quoted"
                    }
                  ]
                },
                #{
                  kind: "text",
                  data: "."
                }
              ]
            }
          ]
        },
        options: #{
          captions: true
        }
      },
      expected: #{
        page: #{
          width: 12240,
          height: 15840,
          landscape: false,
          margins: #{
            top: 1440,
            right: 1440,
            bottom: 1440,
            left: 1440,
            header: 720,
            footer: 720
          },
          header: null,
          footer: null
        },
        body: #[
          #{
            kind: "paragraph",
            style: "normal",
            headingLevel: null,
            list: null,
            align: null,
            line: 259,
            lineRule: "auto",
            indent: #{
              left: 0,
              right: 0,
              firstLine: 0,
              hanging: 0
            },
            keepNext: false,
            keepLines: false,
            borders: null,
            fill: null,
            runs: #[
              #{
                bold: false,
                italic: false,
                strike: false,
                underline: null,
                font: "Aptos",
                size: 24,
                color: null,
                vertAlign: null,
                link: null,
                text: "See {REF qd_cap_flow \\h} and {REF qd_cap_summary \\h}."
              }
            ],
            gapBefore: 0
          },
          #{
            kind: "paragraph",
            style: "caption",
            headingLevel: null,
            list: null,
            align: null,
            line: 259,
            lineRule: "auto",
            indent: #{
              left: 0,
              right: 0,
              firstLine: 0,
              hanging: 0
            },
            keepNext: false,
            keepLines: false,
            borders: null,
            fill: null,
            runs: #[
              #{
                bold: true,
                italic: true,
                strike: false,
                underline: null,
                font: "Aptos",
                size: 18,
                color: null,
                vertAlign: null,
                link: null,
                text: "Figure {SEQ Figure \\* ARABIC}: "
              },
              #{
                bold: false,
                italic: true,
                strike: false,
                underline: null,
                font: "Aptos",
                size: 18,
                color: null,
                vertAlign: null,
                link: null,
                text: "A flow diagram"
              }
            ],
            gapBefore: 200
          },
          #{
            kind: "paragraph",
            style: "caption",
            headingLevel: null,
            list: null,
            align: null,
            line: 259,
            lineRule: "auto",
            indent: #{
              left: 0,
              right: 0,
              firstLine: 0,
              hanging: 0
            },
            keepNext: false,
            keepLines: false,
            borders: null,
            fill: null,
            runs: #[
              #{
                bold: true,
                italic: true,
                strike: false,
                underline: null,
                font: "Aptos",
                size: 18,
                color: null,
                vertAlign: null,
                link: null,
                text: "Table {SEQ Table \\* ARABIC}: "
              },
              #{
                bold: false,
                italic: true,
                strike: false,
                underline: null,
                font: "Aptos",
                size: 18,
                color: null,
                vertAlign: null,
                link: null,
                text: "Summary values"
              }
            ],
            gapBefore: 200
          },
          #{
            kind: "paragraph",
            style: "normal",
            headingLevel: null,
            list: null,
            align: null,
            line: 259,
            lineRule: "auto",
            indent: #{
              left: 360,
              right: 0,
              firstLine: 0,
              hanging: 0
            },
            keepNext: false,
            keepLines: false,
            borders: #{
              left: #{
                style: "single",
                size: 24,
                space: 12,
                color: "8B949E"
              }
            },
            fill: null,
            runs: #[
              #{
                bold: false,
                italic: false,
                strike: false,
                underline: null,
                font: "Aptos",
                size: 24,
                color: "57606A",
                vertAlign: null,
                link: null,
                text: "Figure: Quoted {#quoted}"
              }
            ],
            gapBefore: 320
          },
          #{
            kind: "paragraph",
            style: "normal",
            headingLevel: null,
            list: null,
            align: null,
            line: 259,
            lineRule: "auto",
            indent: #{
              left: 0,
              right: 0,
              firstLine: 0,
              hanging: 0
            },
            keepNext: false,
            keepLines: false,
            borders: null,
            fill: null,
            runs: #[
              #{
                bold: false,
                italic: false,
                strike: false,
                underline: null,
                font: "Aptos",
                size: 24,
                color: null,
                vertAlign: null,
                link: null,
                text: "Jump to "
              },
              #{
                bold: false,
                italic: false,
                strike: false,
                underline: "single",
                font: "Aptos",
                size: 24,
                color: "0563C1",
                vertAlign: null,
                link: "#quoted",
                text: "quoted"
              },
              #{
                bold: false,
                italic: false,
                strike: false,
                underline: null,
                font: "Aptos",
                size: 24,
                color: null,
                vertAlign: null,
                link: null,
                text: "."
              }
            ],
            gapBefore: 320
          }
        ],
        trailingGap: 160
      }
    },

  #{
      name: "render_option_captions_images",
      stage: "callable",
      input: #{
        doc: #{
          blocks: #[
            #{
              kind: "paragraph",
              content: #[
                #{
                  kind: "text",
                  data: "See "
                },
                #{
                  kind: "link",
                  href: "#flow",
                  content: #[
                    #{
                      kind: "image",
                      src: "thumb.png",
                      alt: "thumb",
                      title: ""
                    }
                  ]
                },
                #{
                  kind: "text",
                  data: "."
                }
              ]
            },
            #{
              kind: "paragraph",
              content: #[
                #{
                  kind: "text",
                  data: "Figure: See "
                },
                #{
                  kind: "image",
                  src: "diagram.png",
                  alt: "diagram",
                  title: ""
                },
                #{
                  kind: "text",
                  data: " here {#flow}"
                }
              ]
            }
          ]
        },
        options: #{
          captions: true
        }
      },
      expected: #{
        page: #{
          width: 12240,
          height: 15840,
          landscape: false,
          margins: #{
            top: 1440,
            right: 1440,
            bottom: 1440,
            left: 1440,
            header: 720,
            footer: 720
          },
          header: null,
          footer: null
        },
        body: #[
          #{
            kind: "paragraph",
            style: "normal",
            headingLevel: null,
            list: null,
            align: null,
            line: 259,
            lineRule: "auto",
            indent: #{
              left: 0,
              right: 0,
              firstLine: 0,
              hanging: 0
            },
            keepNext: false,
            keepLines: false,
            borders: null,
            fill: null,
            runs: #[
              #{
                bold: false,
                italic: false,
                strike: false,
                underline: null,
                font: "Aptos",
                size: 24,
                color: null,
                vertAlign: null,
                link: null,
                text: "See {REF qd_cap_flow \\h}."
              }
            ],
            gapBefore: 0
          },
          #{
            kind: "paragraph",
            style: "caption",
            headingLevel: null,
            list: null,
            align: null,
            line: 259,
            lineRule: "auto",
            indent: #{
              left: 0,
              right: 0,
              firstLine: 0,
              hanging: 0
            },
            keepNext: false,
            keepLines: false,
            borders: null,
            fill: null,
            runs: #[
              #{
                bold: true,
                italic: true,
                strike: false,
                underline: null,
                font: "Aptos",
                size: 18,
                color: null,
                vertAlign: null,
                link: null,
                text: "Figure {SEQ Figure \\* ARABIC}: "
              },
              #{
                bold: false,
                italic: true,
                strike: false,
                underline: null,
                font: "Aptos",
                size: 18,
                color: null,
                vertAlign: null,
                link: null,
                text: "See diagram here"
              }
            ],
            gapBefore: 200
          }
        ],
        trailingGap: 160
      }
    }
];
