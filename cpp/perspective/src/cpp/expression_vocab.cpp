// ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
// ┃ ██████ ██████ ██████       █      █      █      █      █ █▄  ▀███ █       ┃
// ┃ ▄▄▄▄▄█ █▄▄▄▄▄ ▄▄▄▄▄█  ▀▀▀▀▀█▀▀▀▀▀ █ ▀▀▀▀▀█ ████████▌▐███ ███▄  ▀█ █ ▀▀▀▀▀ ┃
// ┃ █▀▀▀▀▀ █▀▀▀▀▀ █▀██▀▀ ▄▄▄▄▄ █ ▄▄▄▄▄█ ▄▄▄▄▄█ ████████▌▐███ █████▄   █ ▄▄▄▄▄ ┃
// ┃ █      ██████ █  ▀█▄       █ ██████      █      ███▌▐███ ███████▄ █       ┃
// ┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┫
// ┃ Copyright (c) 2017, the Perspective Authors.                              ┃
// ┃ ╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌ ┃
// ┃ This file is part of the Perspective library, distributed under the terms ┃
// ┃ of the [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0). ┃
// ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛

#include <perspective/expression_vocab.h>

namespace perspective {

t_expression_vocab::t_expression_vocab() {
    // Allocate 4096 bytes per page
    m_max_vocab_size = 64 * 64;

    // Always start with one vocab
    allocate_new_vocab();
}

const char*
t_expression_vocab::intern(const char* str) {
    std::size_t bytelength = strlen(str);

    if (m_current_vocab_size + bytelength + 1 > m_max_vocab_size) {
        allocate_new_vocab();
    }

    m_current_vocab_size += bytelength + 1;
    t_vocab& current_vocab = m_vocabs[0];
    t_uindex interned_idx = current_vocab.get_interned(str);
    return current_vocab.unintern_c(interned_idx);
}

const char*
t_expression_vocab::intern(const std::string& str) {
    return intern(str.c_str());
}

void
t_expression_vocab::clear() {
    m_vocabs.clear();
    allocate_new_vocab();
}

const char*
t_expression_vocab::get_empty_string() const {
    return m_empty_string.c_str();
}

void
t_expression_vocab::pprint() const {
    for (const auto& vocab : m_vocabs) {
        vocab.pprint_vocabulary();
    }
}

void
t_expression_vocab::allocate_new_vocab() {
    t_vocab vocab;
    vocab.init(false);
    vocab.reserve(m_max_vocab_size, 64);
    m_vocabs.insert(m_vocabs.begin(), std::move(vocab));
    m_current_vocab_size = 0;
}

} // end namespace perspective