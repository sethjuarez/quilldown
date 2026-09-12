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
          code: "x^2 + y^2\n",
          kind: "code_block",
          language: "math"
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
