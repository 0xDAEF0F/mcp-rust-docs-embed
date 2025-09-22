use mcp_rust_docs_embed::chunks::{ChunkKind, typescript::extract_typescript_chunks};

#[test]
fn test_typescript_primitives_chunking() {
   let typescript_code = r#"import { Request, Response } from 'express';
import * as fs from 'fs';
import React from 'react';

/**
 * User interface representing a person in the system
 * @interface User
 */
export interface User {
    id: number;
    name: string;
    email: string;
    roles: Role[];
    createdAt: Date;
    metadata?: Record<string, unknown>;
}

/**
 * Role enum for authorization
 */
export enum Role {
    Admin = 'ADMIN',
    User = 'USER',
    Guest = 'GUEST',
    Moderator = 'MODERATOR'
}

// Type alias for user permissions
export type Permission = 'read' | 'write' | 'delete' | 'admin';
export type UserPermissions = Record<string, Permission[]>;

// Complex generic type alias
export type ApiResponse<T, E = Error> =
    | { success: true; data: T }
    | { success: false; error: E };

/**
 * Base class for all services
 * @abstract
 */
abstract class BaseService {
    protected readonly logger: Logger;
    protected config: ServiceConfig;

    constructor(logger: Logger, config: ServiceConfig) {
        this.logger = logger;
        this.config = config;
    }

    abstract initialize(): Promise<void>;
    abstract shutdown(): Promise<void>;
}

/**
 * User service for managing user operations
 * @class UserService
 * @extends BaseService
 */
@Injectable()
@Singleton()
export class UserService extends BaseService {
    private users: Map<number, User> = new Map();
    private readonly db: Database;

    constructor(
        logger: Logger,
        config: ServiceConfig,
        @Inject('DATABASE') db: Database
    ) {
        super(logger, config);
        this.db = db;
    }

    /**
     * Initialize the user service
     */
    async initialize(): Promise<void> {
        this.logger.info('Initializing UserService');
        await this.db.connect();
    }

    /**
     * Get a user by ID
     * @param id - The user ID
     * @returns The user or undefined
     */
    async getUser(id: number): Promise<User | undefined> {
        return this.users.get(id);
    }

    /**
     * Create a new user
     */
    async createUser(data: Omit<User, 'id'>): Promise<User> {
        const id = Math.random();
        const user: User = { id, ...data };
        this.users.set(id, user);
        return user;
    }

    async shutdown(): Promise<void> {
        await this.db.disconnect();
    }
}

// Standalone arrow function
export const validateEmail = (email: string): boolean => {
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    return emailRegex.test(email);
};

// Function declaration with generics
export function mapArray<T, R>(
    items: T[],
    mapper: (item: T, index: number) => R
): R[] {
    return items.map(mapper);
}

// Async function with error handling
export async function fetchUserData(
    userId: number,
    options?: FetchOptions
): Promise<ApiResponse<User>> {
    try {
        const response = await fetch(`/api/users/${userId}`, options);
        const user = await response.json();
        return { success: true, data: user };
    } catch (error) {
        return { success: false, error: error as Error };
    }
}

// Constants and configuration
export const API_VERSION = '2.0.0';
export const MAX_RETRIES = 3;
export const DEFAULT_TIMEOUT = 5000;

export const CONFIG = {
    apiUrl: process.env.API_URL || 'http://localhost:3000',
    debug: process.env.DEBUG === 'true',
    features: {
        auth: true,
        logging: true,
        metrics: false
    }
} as const;

// React functional component with hooks
export const UserProfile: React.FC<{ user: User }> = ({ user }) => {
    const [isEditing, setIsEditing] = React.useState(false);
    const [formData, setFormData] = React.useState(user);

    React.useEffect(() => {
        console.log('User data changed:', user);
    }, [user]);

    const handleSubmit = React.useCallback(async (e: React.FormEvent) => {
        e.preventDefault();
        await updateUser(formData);
        setIsEditing(false);
    }, [formData]);

    return (
        <div className="user-profile">
            <h1>{user.name}</h1>
            <p>{user.email}</p>
        </div>
    );
};

// Namespace declaration
namespace Utils {
    export function formatDate(date: Date): string {
        return date.toISOString();
    }

    export interface Logger {
        info(message: string): void;
        error(message: string, error?: Error): void;
        debug(message: string, data?: unknown): void;
    }

    export class ConsoleLogger implements Logger {
        info(message: string): void {
            console.log(`[INFO] ${message}`);
        }

        error(message: string, error?: Error): void {
            console.error(`[ERROR] ${message}`, error);
        }

        debug(message: string, data?: unknown): void {
            if (CONFIG.debug) {
                console.debug(`[DEBUG] ${message}`, data);
            }
        }
    }
}

// Module declaration
declare module 'custom-module' {
    export interface CustomConfig {
        enabled: boolean;
        options: Record<string, any>;
    }

    export function initialize(config: CustomConfig): void;
}

// Decorator functions
function Injectable() {
    return function <T extends { new(...args: any[]): {} }>(constructor: T) {
        return class extends constructor {
            injected = true;
        };
    };
}

function Singleton() {
    return function <T extends { new(...args: any[]): {} }>(constructor: T) {
        let instance: T;
        return class extends constructor {
            constructor(...args: any[]) {
                if (!instance) {
                    super(...args);
                    instance = this as any;
                }
                return instance;
            }
        };
    };
}

// Complex generic constraints
interface Repository<T extends { id: number }> {
    find(id: number): Promise<T | undefined>;
    findAll(): Promise<T[]>;
    create(data: Omit<T, 'id'>): Promise<T>;
    update(id: number, data: Partial<T>): Promise<T>;
    delete(id: number): Promise<void>;
}

// Conditional types
type IsArray<T> = T extends any[] ? true : false;
type ElementType<T> = T extends (infer E)[] ? E : T;
type PromiseType<T> = T extends Promise<infer U> ? U : T;

// Mapped types
type Readonly<T> = {
    readonly [P in keyof T]: T[P];
};

type Nullable<T> = {
    [P in keyof T]: T[P] | null;
};

// Union and intersection types
type Status = 'pending' | 'approved' | 'rejected';
type Timestamped = { createdAt: Date; updatedAt: Date };
type UserWithTimestamps = User & Timestamped;

// Index signatures and utility types
interface StringMap {
    [key: string]: string;
}

type PartialUser = Partial<User>;
type RequiredUser = Required<User>;
type UserKeys = keyof User;

// JSDoc comments for better documentation
/**
 * Calculate the factorial of a number
 * @param {number} n - The input number
 * @returns {number} The factorial result
 * @example
 * factorial(5) // returns 120
 * @throws {Error} If n is negative
 */
function factorial(n: number): number {
    if (n < 0) throw new Error('Negative input not allowed');
    if (n <= 1) return 1;
    return n * factorial(n - 1);
}

// Template literal types
type HttpMethod = 'GET' | 'POST' | 'PUT' | 'DELETE';
type EndpointPath = `/api/${string}`;
type RouteDefinition = `${HttpMethod} ${EndpointPath}`;

// Assertion functions and type guards
function isUser(value: unknown): value is User {
    return (
        typeof value === 'object' &&
        value !== null &&
        'id' in value &&
        'name' in value &&
        'email' in value
    );
}

function assertIsUser(value: unknown): asserts value is User {
    if (!isUser(value)) {
        throw new Error('Value is not a User');
    }
}

// Async generator function
async function* userGenerator(): AsyncGenerator<User, void, unknown> {
    const users = await fetchAllUsers();
    for (const user of users) {
        yield user;
    }
}

// This is a standalone comment block
// that describes some important system behavior.
// It spans multiple lines and should be treated
// as its own chunk since it's not attached to any code.

// Another standalone comment section
// with implementation notes

/* Block comment style
 * with multiple lines
 * describing architecture decisions
 */
"#;

   let mut chunks = extract_typescript_chunks(typescript_code)
      .unwrap()
      .into_iter();

   assert_eq!(chunks.next().unwrap().kind, ChunkKind::Interface); // User
   assert_eq!(chunks.next().unwrap().kind, ChunkKind::Enum); // Role
   assert_eq!(chunks.next().unwrap().kind, ChunkKind::TypeAlias); // Permission
   assert_eq!(chunks.next().unwrap().kind, ChunkKind::TypeAlias); // UserPermissions
   assert_eq!(chunks.next().unwrap().kind, ChunkKind::TypeAlias); // ApiResponse
   assert_eq!(chunks.next().unwrap().kind, ChunkKind::Comment); // BaseService comment
   assert_eq!(chunks.next().unwrap().kind, ChunkKind::Class); // UserService
   assert_eq!(chunks.next().unwrap().kind, ChunkKind::Const); // validateEmail
   assert_eq!(chunks.next().unwrap().kind, ChunkKind::Function); // mapArray
   assert_eq!(chunks.next().unwrap().kind, ChunkKind::Function); // fetchUserData
   assert_eq!(chunks.next().unwrap().kind, ChunkKind::Const); // API_VERSION
   assert_eq!(chunks.next().unwrap().kind, ChunkKind::Const); // MAX_RETRIES
   assert_eq!(chunks.next().unwrap().kind, ChunkKind::Const); // DEFAULT_TIMEOUT
   assert_eq!(chunks.next().unwrap().kind, ChunkKind::Const); // CONFIG
   assert_eq!(chunks.next().unwrap().kind, ChunkKind::Const); // UserProfile
   assert_eq!(chunks.next().unwrap().kind, ChunkKind::Comment); // Namespace comment
   assert_eq!(chunks.next().unwrap().kind, ChunkKind::Comment); // Module comment
   assert_eq!(chunks.next().unwrap().kind, ChunkKind::Function); // Injectable
   assert_eq!(chunks.next().unwrap().kind, ChunkKind::Function); // Singleton
   assert_eq!(chunks.next().unwrap().kind, ChunkKind::Interface); // Repository
   assert_eq!(chunks.next().unwrap().kind, ChunkKind::Interface); // StringMap
   assert_eq!(chunks.next().unwrap().kind, ChunkKind::Function); // factorial
   assert_eq!(chunks.next().unwrap().kind, ChunkKind::Function); // isUser
   assert_eq!(chunks.next().unwrap().kind, ChunkKind::Function); // assertIsUser
   assert_eq!(chunks.next().unwrap().kind, ChunkKind::Comment); // userGenerator comment
   assert_eq!(chunks.next().unwrap().kind, ChunkKind::Comment); // Standalone comment block
   assert_eq!(chunks.next().unwrap().kind, ChunkKind::Comment); // Another standalone comment
   assert_eq!(chunks.next().unwrap().kind, ChunkKind::Comment); // Block comment
}

#[test]
fn test_decorators_preserved() {
   let code = r#"
/**
 * Service documentation
 */
@Injectable()
@Singleton()
export class MyService {
    constructor() {}
}
"#;

   let chunks = extract_typescript_chunks(code).unwrap();
   assert_eq!(chunks.len(), 1, "Should extract one chunk");

   let chunk = &chunks[0];
   assert_eq!(chunk.kind, ChunkKind::Class);
   assert!(chunk.content.contains("@Injectable()"));
   assert!(chunk.content.contains("@Singleton()"));
   assert!(chunk.content.contains("* Service documentation"));
}

#[test]
fn test_standalone_comments() {
   let code = r#"
import { something } from 'module';

// This is a standalone comment section
// that should be its own chunk
// with multiple lines

export function myFunction() {
    return 42;
}
"#;

   let chunks = extract_typescript_chunks(code).unwrap();

   let comment_chunk = chunks.iter().find(|c| c.kind == ChunkKind::Comment);
   assert!(comment_chunk.is_some(), "Should extract standalone comment");
   assert!(
      comment_chunk
         .unwrap()
         .content
         .contains("standalone comment section")
   );

   let function_chunk = chunks.iter().find(|c| c.kind == ChunkKind::Function);
   assert!(function_chunk.is_some(), "Should extract function");
}

#[test]
fn test_const_exports() {
   let code = r#"
export const API_VERSION = '1.0.0';
export const CONFIG = {
    url: 'http://localhost',
    timeout: 5000
};

const privateConst = 'not exported';
"#;

   let chunks = extract_typescript_chunks(code).unwrap();

   let const_chunks: Vec<_> = chunks
      .iter()
      .filter(|c| c.kind == ChunkKind::Const)
      .collect();

   assert_eq!(const_chunks.len(), 2, "Should extract only exported consts");
   assert!(const_chunks[0].content.contains("API_VERSION"));
   assert!(const_chunks[1].content.contains("CONFIG"));
}

#[test]
fn test_type_aliases() {
   let code = r#"
export type Status = 'active' | 'inactive' | 'pending';

type PrivateType = string | number;

export type GenericType<T> = {
    value: T;
    timestamp: Date;
};
"#;

   let chunks = extract_typescript_chunks(code).unwrap();

   let type_chunks: Vec<_> = chunks
      .iter()
      .filter(|c| c.kind == ChunkKind::TypeAlias)
      .collect();

   assert_eq!(type_chunks.len(), 2, "Should extract exported type aliases");
   assert!(type_chunks[0].content.contains("Status"));
   assert!(type_chunks[1].content.contains("GenericType"));
}

#[test]
fn test_arrow_functions() {
   let code = r#"
export const myArrowFunc = (x: number): number => {
    return x * 2;
};

export const asyncArrow = async (url: string) => {
    const response = await fetch(url);
    return response.json();
};
"#;

   let chunks = extract_typescript_chunks(code).unwrap();
   assert_eq!(chunks.len(), 2, "Should extract arrow functions as consts");
   assert!(chunks.iter().all(|c| c.kind == ChunkKind::Const));
}

#[test]
fn test_react_component() {
   let code = r#"
import React from 'react';

/**
 * User profile component
 */
export const UserProfile: React.FC<{ name: string }> = ({ name }) => {
    return <div>Hello {name}</div>;
};
"#;

   let chunks = extract_typescript_chunks(code).unwrap();

   let component = chunks.iter().find(|c| c.content.contains("UserProfile"));
   assert!(component.is_some(), "Should extract React component");
   assert_eq!(component.unwrap().kind, ChunkKind::Const);
   assert!(
      component
         .unwrap()
         .content
         .contains("* User profile component")
   );
}
