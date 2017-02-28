#!/usr/bin/env PYTHON3

from collections import namedtuple


Condition = namedtuple('Condition', 'state tape_symbol')
Action = namedtuple('Action', 'new_symbol movement next_state')

class Language:

    def __init__(self, func, empty_symbol):
        self.func = func
        self.empty_symbol = empty_symbol

        # Make sure the empty symbol isn't the language, to avoid confusion
        if self.contains(empty_symbol):
            raise ValueError("Empty symbol cannot be in the language")

    def contains(self, symbol):
        return self.func(symbol)

ASCII = Language(lambda c: ord(c) < 128, 'ε')

class TuringMachine:

    def __init__(self, accepting_states, initial_state, language=ASCII):
        self.accepting_states = set(accepting_states)
        self.language = language
        self.initial_state = initial_state
        self.instructions = dict()

    """
    Checks if the given symbol is in this machine's language. If not, raises a ValueError.
    Also raises a ValueError if the length of the supposed symbol is not 1.
    """
    def __check_symbol(self, symbol):
        if len(symbol) != 1:
            raise ValueError("Not a single character: " + symbol)
        if not self.language.contains(symbol):
            raise ValueError("Symbol not in language: " + symbol)

    """
    Adds the given condition/action pair to this machine's instruction set.
    """
    def add_instruction(self, condition, action):
        # Check the validity of both symbols
            self.__check_symbol(condition.tape_symbol)
            self.__check_symbol(action.new_symbol)
            self.instructions[condition] = action  # Add it to the instruction set

    def __apply_action(self, action):
        self.tape[self.head] = action.new_symbol

        # Move the head and extend the tape if needed
        if action.movement == 'L':
            self.head -= 1
            # If we hit the left end, extend the tape with an empty square
            if self.head < 0:
                self.tape.insert(0, self.language.empty_symbol)
        elif action.movement == 'R':
            self.head += 1
            # If we hit the right end, extend the tape with an empty square
            if self.head >= len(self.tape):
                self.tape.append(self.language.empty_symbol)

        self.state = action.next_state

    """
    Run this machine on the given string, returning True if it is in the language, false if not.
    """
    def run(self, s):
        # Initialize values for this run
        self.tape = []
        self.head = 0
        self.state = self.initial_state

        # Fill the tape, checking that each symbol is in the language while we're at it
        for symbol in s:
            self.__check_symbol(symbol)
            self.tape.append(symbol)

        # Loop until we reach an accepting state
        while self.state not in self.accepting_states:
            # Try to find an action for this state/symbol pair. If there is none, halt
            try:
                action = self.instructions[Condition(self.state, self.tape[self.head])]
            except KeyError:
                # No instruction, halt and return false
                return False
            self.__apply_action(action)
        return True

    """
    Run this machine and print ACCEPT or REJECT and the machine's state at halt time.
    """
    def run_and_print(self, s):
        result = self.run(s)
        print("ACCEPT" if result else "REJECT")
        print(self)

    def __str__(self):
        tape = ''.join(self.tape)
        head = ' ' * self.head + '^'
        return "{}\n{}".format(tape, head)



def main():
    machine = TuringMachine(['HALT'], 1)
    machine.add_instruction(Condition(1, 'a'), Action('A', 'R', 2))
    machine.add_instruction(Condition(2, 'b'), Action('B', 'R', 3))
    machine.add_instruction(Condition(3, 'c'), Action('C', 'R', 'HALT'))

    machine.run_and_print('abc')

    machine.run_and_print('cba')


if __name__ == '__main__':
    main()
