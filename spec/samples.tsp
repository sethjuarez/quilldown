namespace Quilldown;

// Auto-derived from the Rust oracle; each promotes an example into a sample.

const Sample_headings = #{
  headings: #{
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
};

const Sample_paragraph_inline = #{
  paragraph_inline: #{
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
};

const Sample_external_link = #{
  external_link: #{
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
};

const Sample_anchor_link = #{
  anchor_link: #{
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
};

const Sample_ordered_list = #{
  ordered_list: #{
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
};

const Sample_ordered_start = #{
  ordered_start: #{
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
};

const Sample_bullet_list = #{
  bullet_list: #{
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
};

const Sample_task_list = #{
  task_list: #{
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
};

const Sample_table = #{
  table: #{
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
};

const Sample_code_block = #{
  code_block: #{
    blocks: #[
      #{
        code: "fn main() {}\n",
        kind: "code_block",
        language: "rust"
      }
    ]
  }
};

const Sample_blockquote = #{
  blockquote: #{
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
};

const Sample_thematic_break = #{
  thematic_break: #{
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
};

const Sample_heading_para = #{
  heading_para: #{
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
};
